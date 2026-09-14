; ModuleID = 'rscs/hello.c'
source_filename = "rscs/hello.c"
target datalayout = "e-m:o-p:32:32-Fi8-f64:32:64-v64:32:64-v128:32:128-a:0:32-n32-S32"
target triple = "armv4t-apple-macosx26.0.0"

%struct.S = type { i32 }

@x = global %struct.S { i32 1 }, align 4
@b = global i16 1, align 2
@a = global i32 0, align 4
@arr = global [2 x i32] zeroinitializer, align 4

; Function Attrs: noinline nounwind optnone ssp
define ptr @h() #0 {
  %1 = alloca [2 x i32], align 4
  %2 = getelementptr inbounds [2 x i32], ptr %1, i32 0, i32 0
  ret ptr %2
}

; Function Attrs: noinline nounwind optnone ssp
define i32 @g() #0 {
  %1 = load i32, ptr @a, align 4
  ret i32 %1
}

; Function Attrs: noinline nounwind optnone ssp
define i32 @f() #0 {
  %1 = alloca %struct.S, align 4
  call void @llvm.memcpy.p0.p0.i32(ptr align 4 %1, ptr align 4 @x, i32 4, i1 false)
  %2 = getelementptr inbounds nuw %struct.S, ptr %1, i32 0, i32 0
  %3 = load i32, ptr %2, align 4
  ret i32 %3
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i32(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i32, i1 immarg) #1

attributes #0 = { noinline nounwind optnone ssp "frame-pointer"="non-leaf-no-reserve" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="arm7tdmi" "target-features"="+armv4t,+soft-float,+strict-align,-aes,-bf16,-d32,-dotprod,-fp-armv8,-fp-armv8d16,-fp-armv8d16sp,-fp-armv8sp,-fp16,-fp16fml,-fp64,-fpregs,-fullfp16,-mve,-mve.fp,-neon,-sha2,-thumb-mode,-vfp2,-vfp2sp,-vfp3,-vfp3d16,-vfp3d16sp,-vfp3sp,-vfp4,-vfp4d16,-vfp4d16sp,-vfp4sp" "use-soft-float"="true" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }

!llvm.module.flags = !{!0, !1, !2}
!llvm.ident = !{!3}

!0 = !{i32 1, !"min_enum_size", i32 4}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"frame-pointer", i32 4}
!3 = !{!"Homebrew clang version 23.1.0"}
