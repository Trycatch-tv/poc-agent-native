(module
  (type (;0;) (func (param i32) (result i32)))
  (type (;1;) (func (param i32 i32) (result i32)))
  (type (;2;) (func (param i32 i32) (result i32)))
  (type (;3;) (func (param i32) (result i32)))
  (type (;4;) (func (param i32 i32) (result i32)))
  (type (;5;) (func (param i32 i32) (result i32)))
  (type (;6;) (func (param i32 i32 i32) (result i32)))
  (type (;7;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;8;) (func (param i32 i32 i32 i32) (result i32)))
  (type (;9;) (func (param i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (import "host" "kv_get" (func (;0;) (type 0)))
  (import "host" "emit_audit" (func (;1;) (type 1)))
  (memory (;0;) 1)
  (global (;0;) (mut i32) i32.const 1024)
  (export "calculate_order" (func 11))
  (export "memory" (memory 0))
  (func (;2;) (type 2) (param i32 i32) (result i32)
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
  (func (;3;) (type 3) (param i32) (result i32)
    (local i32)
    local.get 0
    i32.const 0
    i32.lt_s
    if (result i32) ;; label = @1
      i32.const 0
    else
      local.get 0
      i32.const 10
      i32.mul
      i32.const 100
      i32.div_s
    end
  )
  (func (;4;) (type 4) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    i32.add
  )
  (func (;5;) (type 5) (param i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    call 0
    i32.mul
    i32.const 100
    i32.div_s
  )
  (func (;6;) (type 6) (param i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    i32.add
    local.get 2
    i32.add
  )
  (func (;7;) (type 7) (param i32 i32 i32 i32 i32) (result i32)
    (local i32)
    global.get 0
    local.set 5
    global.get 0
    i32.const 24
    i32.add
    global.set 0
    local.get 5
    i32.const 100
    local.get 4
    call 1
    i32.const 0
    i32.ge_s
    if (result i32) ;; label = @1
      i32.const 0
    else
      i32.const 0
    end
    i32.store
    local.get 5
    local.get 0
    i32.store offset=4
    local.get 5
    local.get 1
    i32.store offset=8
    local.get 5
    local.get 2
    i32.store offset=12
    local.get 5
    local.get 3
    i32.store offset=16
    local.get 5
    local.get 4
    i32.store offset=20
    local.get 5
  )
  (func (;8;) (type 8) (param i32 i32 i32 i32) (result i32)
    (local i32)
    local.get 1
    local.get 2
    call 4
    local.get 1
    local.get 2
    call 4
    local.get 0
    call 5
    local.get 3
    call 9
  )
  (func (;9;) (type 9) (param i32 i32 i32) (result i32)
    (local i32)
    local.get 0
    local.get 1
    local.get 0
    local.get 1
    i32.sub
    call 3
    local.get 2
    local.get 0
    local.get 1
    i32.sub
    local.get 0
    local.get 1
    i32.sub
    call 3
    local.get 2
    call 6
    call 7
  )
  (func (;10;) (type 10) (param i32 i32 i32 i32) (result i32)
    (local i32)
    local.get 3
    i32.const 0
    i32.lt_s
    if (result i32) ;; label = @1
      global.get 0
      local.set 4
      global.get 0
      i32.const 24
      i32.add
      global.set 0
      local.get 4
      i32.const 2
      i32.store
      local.get 4
      i32.const 0
      i32.store offset=4
      local.get 4
      i32.const 0
      i32.store offset=8
      local.get 4
      i32.const 0
      i32.store offset=12
      local.get 4
      i32.const 0
      i32.store offset=16
      local.get 4
      i32.const 0
      i32.store offset=20
      local.get 4
    else
      local.get 0
      local.get 1
      local.get 2
      local.get 3
      call 8
    end
  )
  (func (;11;) (type 11) (param i32 i32 i32 i32 i32 i32) (result i32)
    (local i32)
    local.get 1
    local.get 2
    call 2
    i32.const 0
    i32.lt_s
    local.get 3
    local.get 4
    call 2
    i32.const 0
    i32.lt_s
    i32.or
    if (result i32) ;; label = @1
      global.get 0
      local.set 6
      global.get 0
      i32.const 24
      i32.add
      global.set 0
      local.get 6
      i32.const 1
      i32.store
      local.get 6
      i32.const 0
      i32.store offset=4
      local.get 6
      i32.const 0
      i32.store offset=8
      local.get 6
      i32.const 0
      i32.store offset=12
      local.get 6
      i32.const 0
      i32.store offset=16
      local.get 6
      i32.const 0
      i32.store offset=20
      local.get 6
    else
      local.get 0
      local.get 1
      local.get 2
      call 2
      local.get 3
      local.get 4
      call 2
      local.get 5
      call 10
    end
  )
)
