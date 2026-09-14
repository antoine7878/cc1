; ModuleID = 'rscs/hello.c'
source_filename = "rscs/hello.c"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "arm64-apple-macosx26.0.0"

%struct.S = type { i32 }

@x = global %struct.S { i32 1 }, align 4
@b = global i16 1, align 2
@a = global i32 0, align 4
@arr = global [2 x i32] zeroinitializer, align 4

; Function Attrs: noinline nounwind optnone ssp uwtable(sync)
define ptr @h() #0 {
  %1 = alloca [2 x i32], align 4
  %2 = getelementptr inbounds [2 x i32], ptr %1, i64 0, i64 0
  ret ptr %2
}

; Function Attrs: noinline nounwind optnone ssp uwtable(sync)
define i32 @g() #0 {
  %1 = load i32, ptr @a, align 4
  ret i32 %1
}

; Function Attrs: noinline nounwind optnone ssp uwtable(sync)
define i32 @f() #0 {
  %1 = alloca %struct.S, align 4
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %1, ptr align 4 @x, i64 4, i1 false)
  %2 = getelementptr inbounds nuw %struct.S, ptr %1, i32 0, i32 0
  %3 = load i32, ptr %2, align 4
  ret i32 %3
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #1

attributes #0 = { noinline nounwind optnone ssp uwtable(sync) "frame-pointer"="non-leaf-no-reserve" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+altnzcv,+ccdp,+ccidx,+ccpp,+complxnum,+crc,+dit,+dotprod,+flagm,+fp-armv8,+fp16fml,+fptoint,+fullfp16,+jsconv,+lse,+neon,+pauth,+perfmon,+predres,+ras,+rcpc,+rdm,+sb,+sha2,+sha3,+specrestrict,+ssbs,+v8.1a,+v8.2a,+v8.3a,+v8.4a,+v8a" "tune-cpu"="apple-m5" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }

!llvm.module.flags = !{!0, !1, !2, !3}
!llvm.ident = !{!4}

!0 = !{i32 2, !"SDK Version", [2 x i32] [i32 26, i32 5]}
!1 = !{i32 8, !"PIC Level", i32 2}
!2 = !{i32 7, !"uwtable", i32 1}
!3 = !{i32 7, !"frame-pointer", i32 4}
!4 = !{!"Homebrew clang version 23.1.0"}
