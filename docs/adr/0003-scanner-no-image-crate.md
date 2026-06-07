# ADR-0003: 扫描器不使用 image crate 读取尺寸

**日期:** 2026-06-07
**状态:** 已接受

## 背景

原计划在扫描阶段使用 `image::image_dimensions()` 读取图片尺寸。实测发现该函数在处理某些图片格式时触发 Windows `/GS` 栈缓冲区溢出检测 (`STATUS_STACK_BUFFER_OVERRUN`)，导致应用崩溃。

## 决策

扫描阶段不读取图片尺寸（设为 `None`），尺寸在生成缩略图时通过 `image::open()` 获取。扫描器只使用 `std::fs` 和 `sha2`，不依赖 `image` crate。

## 理由

- `image::image_dimensions()` 内部解码图片头，某些格式（RAW、TIFF）的 C 层解码器存在栈缓冲区溢出
- `catch_unwind` 无法捕获 `/GS` 安全检查失败（非 Rust panic）
- 扫描阶段不需要精确尺寸，缩略图生成时会获取

## 后果

- 扫描更快（不需解码图片头）
- 图片尺寸初始为 `null`，生成缩略图后填充
- `image` crate 仅在 `scanner/thumbnail.rs` 中使用，且包裹在 `catch_unwind` 中
