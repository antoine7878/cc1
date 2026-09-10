; ModuleID = 'rscs/hello.c'
source_filename = "rscs/hello.c"
target datalayout = "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-f64:32:64-f80:32-n8:16:32-S128"
target triple = "i386-pc-linux-gnu"

@.str = private unnamed_addr constant [7 x i8] c"Hello3\00", align 1
@.str.1 = private unnamed_addr constant [7 x i8] c"Hello2\00", align 1
@.str.2 = private unnamed_addr constant [6 x i8] c"Hello\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @f3() #0 {
  ret ptr @.str
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @f2() #0 {
  ret ptr @.str.1
}

; Function Attrs: noinline nounwind optnone uwtable
define dso_local ptr @f1() #0 {
  ret ptr @.str.2
}

attributes #0 = { noinline nounwind optnone uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="i686" "target-features"="+cmov,+cx8,+x87" "tune-cpu"="generic" }

!llvm.module.flags = !{!0, !1, !2, !3, !4, !5}
!llvm.ident = !{!6}

!0 = !{i32 1, !"NumRegisterParameters", i32 0}
!1 = !{i32 1, !"wchar_size", i32 4}
!2 = !{i32 8, !"PIC Level", i32 2}
!3 = !{i32 7, !"PIE Level", i32 2}
!4 = !{i32 7, !"uwtable", i32 2}
!5 = !{i32 7, !"frame-pointer", i32 2}
!6 = !{!"Ubuntu clang version 18.1.3 (1ubuntu1)"}
