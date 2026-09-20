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
        self.assertEqual(got, ["clang", "-m32", "main.o", "-lfoo", "helper.o", "-o", "a.out"])


if __name__ == "__main__":
    unittest.main()
