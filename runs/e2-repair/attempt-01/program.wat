(module
  (type (;0;) (func (param i32) (result i32)))
  (export "calculate_shipping" (func 0))
  (func (;0;) (type 0) (param i32) (result i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if (result i32) ;; label = @1
      i32.const -1
    else
      local.get 0
      i32.const 5000
      i32.ge_s
      if (result i32) ;; label = @2
        i32.const 0
      else
        i32.const 500
      end
    end
  )
)
