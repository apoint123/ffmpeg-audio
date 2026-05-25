#![allow(nonstandard_style)]
#![allow(unnecessary_transmutes)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// C `va_list` 类型在作为函数参数传递时的正确 Rust 类型。
///
/// ## 背景
///
/// C 标准将 `va_list` 定义为“实现定义”的类型，各平台的底层表现形式差异显著，主要分为以下三类：
///
/// - **指针类型**（Windows、macOS aarch64、powerpc64 等）：
///   `va_list` 直接是 `char *` 或类似的不透明指针，`bindgen` 会生成为 `*mut c_char`。
///   此类平台的参数传递无歧义，直接使用 `bindgen` 生成的 `va_list` 类型别名即可。
///
/// - **结构体类型**（aarch64 Linux/Android）：
///   `va_list` 是一个普通的结构体。例如：
///   ```c
///   struct __va_list {
///       void *__ap;
///   };
///   ```
///   `bindgen` 会生成对应的 Rust 结构体并按值传递，同样无歧义。
///
/// - **单元素数组类型**（x86_64 System V ABI、PowerPC32 Linux、s390x 等）：
///   `va_list` 被定义为 `__va_list_tag[1]` 的单元素数组。例如在 x86_64 Linux 上：
///   ```c
///   typedef struct {
///       unsigned int gp_offset;
///       unsigned int fp_offset;
///       void *overflow_arg_area;
///       void *reg_save_area;
///   } __va_list_tag;
///
///   typedef __va_list_tag va_list[1];
///   ```
///   **ABI 退化问题**：在 C 语言中，数组类型作为函数参数时会自动退化为指针。因此，在这些平台上接受
///   `va_list` 参数的函数，其实际的 ABI 参数类型是 `__va_list_tag *`，而非数组本身。
///   此时，`bindgen` 会将函数签名中的参数生成为 `*mut __va_list_tag`，但同时会将 `va_list`
///   的类型别名本身生成为数组 `[__va_list_tag; 1]`。若直接使用 `va_list` 类型别名定义回调函数的参数，
///   就会与 C 函数实际期望的指针类型 `*mut __va_list_tag` 发生不匹配。
///
/// ## 解决方案
///
/// 为了抹平上述 ABI 差异，这里采用条件编译：
/// - **对于数组退化平台**：直接使用退化后的指针类型 `*mut __va_list_tag`；
/// - **对于其他平台**：直接使用 `bindgen` 生成的 `va_list` 类型别名。
///
/// **判断条件依据**：
/// 这里的判断条件与 Rust 标准库 [`core::ffi::VaList`] 的底层实现保持一致：凡是 `VaListInner` 带有
/// `#[rustc_pass_indirectly_in_non_rustic_abis]` 属性的平台（即 x86_64 System V、PowerPC32、s390x
/// 等），C 的 `va_list` 均为数组类型，均需要使用退化后的指针形式。
///
/// 目前项目支持的目标平台中，受影响的主要有：
/// - `x86_64-unknown-linux-gnu` (Linux x86_64)
/// - `x86_64-linux-android` (Android x86_64)
#[cfg(not(all(target_arch = "x86_64", not(windows), not(target_os = "uefi"))))]
pub type VaList = va_list;

#[cfg(all(target_arch = "x86_64", not(windows), not(target_os = "uefi")))]
pub type VaList = *mut __va_list_tag;
