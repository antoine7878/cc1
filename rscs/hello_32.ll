; ModuleID = 'rscs/hello.c'
source_filename = "rscs/hello.c"
target datalayout = "e-m:o-p:32:32-Fi8-f64:32:64-v64:32:64-v128:32:128-a:0:32-n32-S32"
target triple = "armv4t-apple-macosx26.0.0"

; Function Attrs: noinline nounwind optnone ssp
define i32 @main() #0 {
  ; %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  ; store i32 0, ptr %1, align 4
  store i32 40, ptr %2, align 4
  %4 = load i32, ptr %2, align 4
  %5 = add nsw i32 %4, 1
  store i32 %5, ptr %2, align 4
  store i32 %4, ptr %3, align 4
  %6 = load i32, ptr %2, align 4
  %7 = load i32, ptr %3, align 4
  %8 = add nsw i32 %6, %7
  ret i32 %8
}

define i32 @main() {
  %1 = alloca i32
  %2 = alloca i32
  store i32 40, ptr %1
  %3 = load i32, ptr %1
  %4 = add nsw i32 %3, 1
  store i32 %4, ptr %1
  store i32 %3, ptr %2
  %5 = load i32, ptr %1
  %6 = add nsw i32 %5, 1
  store i32 %6, ptr %1
  %7 = load i32, ptr %1
  %8 = load i32, ptr %2
  %9 = add nsw i32 %7, %8
  ret i32 %9
}
attributes #0 = { noinline nounwind optnone ssp "frame-pointer"="non-leaf-no-reserve" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="arm7tdmi" "target-features"="+armv4t,+soft-float,+strict-align,-aes,-bf16,-d32,-dotprod,-fp-armv8,-fp-armv8d16,-fp-armv8d16sp,-fp-armv8sp,-fp16,-fp16fml,-fp64,-fpregs,-fullfp16,-mve,-mve.fp,-neon,-sha2,-thumb-mode,-vfp2,-vfp2sp,-vfp3,-vfp3d16,-vfp3d16sp,-vfp3sp,-vfp4,-vfp4d16,-vfp4d16sp,-vfp4sp" "use-soft-float"="true" }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}

!0 = !{i32 1, !"min_enum_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"frame-pointer", i32 4}
!3 = !{!"Homebrew clang version 23.1.0"}
