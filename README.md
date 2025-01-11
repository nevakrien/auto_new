# `auto_new`

This crate automatically generates `new` and `new_arc` functions for structs using the `#[derive(new)]` macro.

### Why new_arc?
The `new_arc` function is designed to be more efficient than what most people would write by hand in most cases. This is because the rust compiler usually treats:
```rust
Arc::new(MyStruct{...})
```
as two seprate memory allocations. This makes it hard for the optimizer to inline most of what should be inlined. Which is why we use `Arc::new_uninit()` and explictly write each of the feilds.

---

## Usage

To use `#[derive(new)]`, simply annotate your struct with it, and the macro will generate the `new` function for you. Here's an example:

```rust
use auto_new::new;

#[derive(new)]
pub struct MyStruct {
    a: u32,
    b: String,
}

fn main() {
    let instance = MyStruct::new(42, String::from("Hello, world!"));
    println!("MyStruct: {}, {}", instance.a, instance.b);
}
```

The crate is fairly robust and works with all sorts of weird generics and lifetimes without issue.
```rust
use auto_new::new;

#[derive(new,Debug)]
pub struct GenericStruct<'a, T> where T:Copy {
    a: &'a T,
    b: Box<u32>,
}

fn main() {
    let value = 42;
    let instance = GenericStruct::new_arc(&value, Box::new(value));
    println!("{:?}", instance);
}
```

## Features

### Customizing Visibility 

You may not want the new functions to be public.
```rust
#[derive(new)]
#[new_visibility(/*private*/)]
struct MyStruct<'a, T:Debug>(&'a T);
```

Allternativly it might be benificial to use a more complex qualifier.
```rust
#[derive(new)]
#[new_visibility(pub(crate))]
struct MyStruct<'a, T:Debug>(&'a T);
```

### Excluding new_arc for no_std Environments
If you don't need the new_arc function, you can exclude it with the #[no_new_arc] attribute:

```rust
#[derive(new)]
#[no_new_arc]
struct MyStruct<T>(&'a T,&'b T);
```

## Future Work
It might be nice to include more types like Box,Rc,RefCell,Mutex etc.
There might also be a way to allow the user to write the new function and then "automagically" generate functions for all types based on that.

This is already somewhat achived by [makeit](https://github.com/estebank/makeit)