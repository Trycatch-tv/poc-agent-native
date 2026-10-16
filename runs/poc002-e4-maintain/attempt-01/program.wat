(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (memory (;0;) 1)
  (global (;0;) (mut i32) i32.const 1024)
  (export "calculate_order" (func 2))
  (export "memory" (memory 0))
  (func (;0;) (type 0) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    i32.const 0
    i32.lt_s
    local.get 1
    i32.const 0
    i32.lt_s
    i32.or
    if (result i32) ;; label = @1
      i32.const -1
    else
      local.get 0
      local.get 1
      i32.mul
    end
  )
  (func (;1;) (type 1) (param i32) (result i32)
    (local i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if (result i32) ;; label = @1
      i32.const -1
    else
      local.get 0
      i32.const 15
      i32.mul
      i32.const 100
      i32.div_s
    end
  )
  (func (;2;) (type 2) (param i32 i32 i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call 0
    i32.const 0
    i32.lt_s
    local.get 2
    local.get 3
    call 0
    i32.const 0
    i32.lt_s
    i32.or
    if (result i32) ;; label = @1
      global.get 0
      local.set 5
      global.get 0
      i32.const 20
      i32.add
      global.set 0
      local.get 5
      i32.const 1
      i32.store
      local.get 5
      i32.const 0
      i32.store offset=4
      local.get 5
      i32.const 0
      i32.store offset=8
      local.get 5
      i32.const 0
      i32.store offset=12
      local.get 5
      i32.const 0
      i32.store offset=16
      local.get 5
    else
      local.get 4
      i32.const 0
      i32.lt_s
      if (result i32) ;; label = @2
        global.get 0
        local.set 5
        global.get 0
        i32.const 20
        i32.add
        global.set 0
        local.get 5
        i32.const 2
        i32.store
        local.get 5
        i32.const 0
        i32.store offset=4
        local.get 5
        i32.const 0
        i32.store offset=8
        local.get 5
        i32.const 0
        i32.store offset=12
        local.get 5
        i32.const 0
        i32.store offset=16
        local.get 5
      else
        i32.const 1000000
        local.get 0
        local.get 1
        call 0
        local.get 2
        local.get 3
        call 0
        i32.add
        local.get 0
        local.get 1
        call 0
        local.get 2
        local.get 3
        call 0
        i32.add
        call 1
        i32.add
        i32.lt_s
        if (result i32) ;; label = @3
          global.get 0
          local.set 5
          global.get 0
          i32.const 20
          i32.add
          global.set 0
          local.get 5
          i32.const 3
          i32.store
          local.get 5
          i32.const 0
          i32.store offset=4
          local.get 5
          i32.const 0
          i32.store offset=8
          local.get 5
          i32.const 0
          i32.store offset=12
          local.get 5
          i32.const 0
          i32.store offset=16
          local.get 5
        else
          global.get 0
          local.set 5
          global.get 0
          i32.const 20
          i32.add
          global.set 0
          local.get 5
          i32.const 0
          i32.store
          local.get 5
          local.get 0
          local.get 1
          call 0
          local.get 2
          local.get 3
          call 0
          i32.add
          i32.store offset=4
          local.get 5
          local.get 0
          local.get 1
          call 0
          local.get 2
          local.get 3
          call 0
          i32.add
          call 1
          i32.store offset=8
          local.get 5
          local.get 4
          i32.store offset=12
          local.get 5
          local.get 0
          local.get 1
          call 0
          local.get 2
          local.get 3
          call 0
          i32.add
          local.get 0
          local.get 1
          call 0
          local.get 2
          local.get 3
          call 0
          i32.add
          call 1
          i32.add
          local.get 4
          i32.add
          i32.store offset=16
          local.get 5
        end
      end
    end
  )
)
