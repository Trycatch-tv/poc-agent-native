(module
  (type (;0;) (func (param i32) (result i32)))
  (type (;1;) (func (param i32 i32) (result i32)))
  (export "calculate_shipping" (func 0))
  (export "calculate_discount" (func 1))
  (func (;0;) (type 0) (param i32) (result i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if (result i32) ;; label = @1
      i32.const -1
    else
      local.get 0
      i32.const 4000
      i32.ge_s
      if (result i32) ;; label = @2
        i32.const 0
      else
        i32.const 600
      end
    end
  )
  (func (;1;) (type 1) (param i32 i32) (result i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if (result i32) ;; label = @1
      i32.const -1
    else
      local.get 1
      i32.const 0
      i32.eq
      if (result i32) ;; label = @2
        i32.const 0
      else
        local.get 0
        i32.const 10
        i32.div_s
      end
    end
  )
)
