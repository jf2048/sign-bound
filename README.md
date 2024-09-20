# sign-bound &emsp;  [![Latest Version]][crates.io] [![Documentation]][docs]

[Documentation]: https://docs.rs/sign-bound/badge.svg
[docs]: https://docs.rs/sign-bound
[Latest Version]: https://img.shields.io/crates/v/sign-bound.svg
[crates.io]: https://crates.io/crates/sign-bound

Signed integer types for Rust that are bounded to be either positive or
negative. The API is analogous to the built-in [`NonZero`] types:

- `PositiveI8`, `NegativeI8`
- `PositiveI16`, `NegativeI16`
- `PositiveI32`, `NegativeI32`
- `PositiveI64`, `NegativeI64`
- `PositiveIsize`, `NegativeIsize`

The types are all memory-layout optimized, so for example `Option<PositiveI32>`
and `Option<NegativeI32>` are both the same size as `i32`. Using additional
variants in an enum can also have some space benefits.

```rust
enum MyEnum {
    A(PositiveI16),
    B,
    C,
    D,
}
assert_eq!(size_of::<MyEnum>(), size_of::<PositiveI16>());
```

Note that due to the implementation details of this crate, the space
optimization for any type will not occur if there are more than 128 additional
enum variants.

`Option<PositiveIsize>` is particularly useful as a space-efficient optional
`Vec` index, since Rust's `Vec` structure is
[limited](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push) to
`isize::MAX` entries.

[`NonZero`]: (https://doc.rust-lang.org/std/num/struct.NonZero.html)
