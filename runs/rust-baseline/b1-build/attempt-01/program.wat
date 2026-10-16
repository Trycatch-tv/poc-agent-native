(module
  (type (;0;) (func (param i32) (result i32)))
  (memory (;0;) 16)
  (global (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048576)
  (global (;2;) i32 i32.const 1048576)
  (export "memory" (memory 0))
  (export "calculate_shipping" (func 0))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (func (;0;) (type 0) (param i32) (result i32)
    i32.const 0
    i32.const 500
    local.get 0
    i32.const 4999
    i32.gt_u
    select
    i32.const -1
    local.get 0
    i32.const -1
    i32.gt_s
    select
  )
)
