import importlib.util
import sys
import unittest
from argparse import Namespace
from importlib.machinery import SourceFileLoader
from pathlib import Path


def load_fcc():
    path = Path(__file__).parents[1] / "fcc"
    loader = SourceFileLoader("fcc", str(path))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    module = importlib.util.module_from_spec(spec)
    sys.modules[loader.name] = module
    loader.exec_module(module)
    return module


class LinkItemsTest(unittest.TestCase):
    def test_preserves_input_and_library_order(self):
        fcc = load_fcc()
        got = fcc.link_items(["main.c", "-L", "lib", "-lfoo", "helper.o", "-D", "NAME=1", "-Iinc", "-o", "a.out"])
        self.assertEqual(
            [(item.value, item.is_input) for item in got],
            [("main.c", True), ("-Llib", False), ("-lfoo", False), ("helper.o", True)],
        )

    def test_link_substitutes_compiled_objects_in_place(self):
        fcc = load_fcc()
        args = Namespace(
            strip=False,
            outfile="a.out",
            link_items=fcc.link_items(["main.c", "-lfoo", "helper.o"]),
        )
        got = fcc.link_argv(args, {"main.c": ["main.o"], "helper.o": ["helper.o"]})
        self.assertEqual(got, ["clang", "--target=i686-linux-gnu", "main.o", "-lfoo", "helper.o", "-o", "a.out"])

    def test_link_forwards_strip_option(self):
        fcc = load_fcc()
        args = Namespace(
            strip=True,
            outfile="a.out",
            link_items=fcc.link_items(["main.o", "-s"]),
        )
        got = fcc.link_argv(args, {"main.o": ["main.o"]})
        self.assertEqual(got, ["clang", "--target=i686-linux-gnu", "-s", "main.o", "-o", "a.out"])


class StageArgvTest(unittest.TestCase):
    def test_preprocess_targets_i386(self):
        fcc = load_fcc()
        args = Namespace(defines=["N=1"], undefines=[], includes=["inc"])
        got = fcc.preprocess_argv(fcc.Step(fcc.Stage.PREPROCESS, "a.c", "a.i"), args)
        self.assertEqual(got, ["clang", "-E", "-std=c89", "--target=i686-linux-gnu", "-DN=1", "-Iinc", "-o", "a.i", "a.c"])

    def test_assemble_targets_i386(self):
        fcc = load_fcc()
        got = fcc.assemble_argv(fcc.Step(fcc.Stage.ASSEMBLE, "a.s", "a.o"), Namespace())
        self.assertEqual(got, ["i686-linux-gnu-as", "--32", "-o", "a.o", "a.s"])


if __name__ == "__main__":
    unittest.main()
