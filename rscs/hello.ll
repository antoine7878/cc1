target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"


define i32 @fc() {

  ret i32 1
}

define void @f() {

  %1 = load ptr, ptr @fc, align 8
  %2 = call ptr %1()
  ret void
}
