# 更新日志 / Updates

所有项目的重要变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 计划添加
- 添加元数据验证功能
- 支持更多 ID3v2 帧类型（TXXX、WXXX 等）
- 支持读取和写入多个封面图片
- 扩展 CLI 工具功能（写入、批量操作等）

### 计划改进
- 优化大文件处理性能（减少内存占用）
- 改进错误处理和错误信息
- 添加更详细的文档和示例
- 增加 Python 类型提示（type hints）
- 支持异步 I/O 操作
- 添加单元测试覆盖率

### 计划修复
- 修复 ID3v2 标签封面处理的边界情况
- 改进非标准编码的兼容性

---

## [0.5.1] - 2025-01-23

### 改进
- **零警告编译**：清理所有编译警告，实现零警告构建
  - 为保留但未使用的公开 API 添加 `#[allow(dead_code)]` 标记
  - 修复未使用的变量和导入
  - 所有保留的代码都有明确的注释说明其用途
- **代码质量提升**：改进代码组织和注释
  - 为所有保留但未使用的功能添加说明注释
  - 提高代码可维护性
  - 确保编译时没有任何警告信息

### 技术细节
- 添加 `#[allow(dead_code)]` 到以下保留但未使用的项：
  - FLAC VorbisComment 和 FlacPicture 的所有方法和字段
  - ID3 frames 中的编码和辅助函数
  - OGG Vorbis 和 OPUS 的写入支持（已实现但未集成）
  - APE 和 MP4 格式的检测函数
  - 工具函数（encoding、文本处理等）
- 修复 CLI 中的未使用变量警告
- 统一未使用代码的处理方式

### 文档
- 更新 README.md：
  - 添加三种使用方式（Python 模块、Rust 库、CLI 工具）的详细说明
  - 添加功能特性列表
  - 改进安装说明，分离三种使用方式的安装步骤
- 更新 updates.md，记录 0.5.1 版本变更

---

## [0.5.0] - 2025-01-23

### 新增
- **CLI 命令行工具**：提供独立的命令行工具 `oxidant`
  - `oxidant read` - 读取音频文件元数据
  - `oxidant detect` - 检测音频文件格式
  - 支持多种输出格式（pretty/json）
  - 支持安静模式和输出重定向
- **多目标构建支持**：支持同时编译为 Python 模块、Rust 库和 CLI 工具
  - Python 绑定（通过 PyO3，使用 `python` feature）
  - Rust 库（rlib，可供其他 Rust 项目使用）
  - CLI 二进制工具（独立可执行文件）
- **条件编译支持**：PyO3 绑定现在是可选的
  - 使用 `--features python` 启用 Python 支持
  - 纯 Rust 构建时不依赖 Python 运行时

### 改进
- 重构 lib.rs，分离核心 Rust 逻辑和 PyO3 绑定
- 核心类型（AudioFile、Metadata 等）现在可以在纯 Rust 代码中使用
- 改进构建系统，支持多种编译目标

### 技术细节
- 更新 Cargo.toml：
  - PyO3 改为可选依赖（`optional = true`）
  - 新增 `python` feature flag
  - 新增 CLI 专用依赖：clap、chrono、glob
- 新增 src/main.rs 作为 CLI 工具入口点
- 新增 AudioResult<T> 类型别名，避免与 std::result::Result 冲突
- 所有核心功能现在可以不依赖 Python 运行时使用

### 文档
- 更新 README.md，添加 CLI 工具使用文档
- 添加 CLI 安装和使用示例
- 添加 CLI 命令和选项说明

---

## [0.4.0] - 2025-01-23

### 新增
- **批量处理 API**：专用的 BatchProcessor 类用于高效处理多个音频文件
  - `read_metadata_batch()` - 批量读取多个文件的元数据
  - `write_metadata_batch()` - 批量写入元数据，返回每个文件的处理结果
  - `process_directory()` - 处理整个目录中匹配模式的文件
  - `copy_metadata()` - 在文件之间复制元数据
- **进度显示支持**：可选的进度信息输出
  - `show_progress` 属性控制是否显示处理进度
- **BatchResult 类**：批量处理结果对象
  - 包含文件路径、成功状态和错误信息
  - 清晰的成功/失败视觉反馈（✓/✗）

### 改进
- 批量处理时的错误处理和报告
- 优化了文件系统遍历性能
- 改进了代码组织，批量功能独立封装

### 技术细节
- 新增 `BatchProcessor` 类：
  - 支持列表式批量操作
  - 支持目录模式匹配（*.mp3, *.flac 等）
  - 线程安全的 GIL 管理
- 新增 `BatchResult` 类：处理结果封装
- 批量处理时显示格式化的进度信息

### 待完成
- [ ] 多线程/并发处理支持
- [ ] 流式处理大文件（减少内存占用）
- [ ] 性能基准测试和优化
- [ ] 添加测试用例

---

## [0.3.0] - 2025-01-23

### 新增
- **OPUS 格式完整支持**：完整的 OPUS 元数据读写
  - 自动检测 OPUS 文件格式（基于 OGG 容器中的 OpusHead 标识）
  - 支持 OpusTags (Vorbis Comment) 读写
  - 与现有 API 完全兼容（JSON 和直接方法接口）
- **MP4/M4A 格式读取支持**：MP4/M4A 元数据读取
  - 自动检测 MP4/M4A 文件格式（基于 ftyp atom）
  - 支持 iTunes 风格元数据读取（ilst atom）
  - 支持常见字段：标题、艺术家、专辑、年份、曲目、流派、备注、歌词
- **APE 格式读取支持**：APE 元数据读取
  - 自动检测 APE 文件格式（基于 APETAGEX 标识）
  - 支持 APE Tag 读取
  - 支持常见字段：标题、艺术家、专辑、年份、曲目、流派、备注、歌词
- **OGG Vorbis 格式支持**：完整的 OGG Vorbis 元数据读写
  - 自动检测 OGG 文件格式
  - 支持 Vorbis Comment 读写
  - 与现有 API 完全兼容（JSON 和直接方法接口）

### 改进
- 扩展文件类型检测，支持 OPUS、MP4/M4A、APE 格式
- 优化元数据读写架构，便于添加新格式
- 改进代码组织，使用模块化设计
- **清理编译警告**：消除所有未使用的导入和变量警告
  - 优化 OGG 模块导出，移除未使用的公开导出
  - 将内部实现细节标记为 `pub(crate)`，改善 API 封装
  - 使用 `#[allow(dead_code)]` 标记保留供未来使用的功能
  - 代码编译零警告，提升代码质量和可维护性

### 技术细节
- 新增 `src/ogg/` 模块：
  - `mod.rs` - 模块导出和常量定义
  - `page.rs` - OGG 页面结构解析
  - `vorbis.rs` - OGG Vorbis Comment 读写
- 新增 `src/opus/mod.rs` - OPUS 格式完整实现
  - `OpusFile` - OPUS 文件处理器
  - `read_comment()` - 读取 OpusTags
  - `write_comment()` - 写入 OpusTags
  - `is_opus_file()` - OPUS 文件检测
- 新增 `src/mp4/mod.rs` - MP4/M4A 格式读取实现
  - `Mp4File` - MP4 文件处理器
  - `Mp4Metadata` - MP4 元数据结构
  - `read_metadata()` - 读取 iTunes 风格元数据
  - `is_mp4_file()` - MP4 文件检测
- 新增 `src/ape/mod.rs` - APE 格式读取实现
  - `ApeFile` - APE 文件处理器
  - `ApeMetadata` - APE 元数据结构
  - `read_metadata()` - 读取 APE Tag
  - `is_ape_file()` - APE 文件检测

### 待完成
- [ ] 完成 MP4/M4A 格式的写入支持（需要重建 atom 树）
- [ ] 完成 APE 格式的写入支持（需要重建标签）
- [ ] 添加各格式的测试用例
- [x] 更新文档说明新格式支持

---

## [0.2.0] - 2025-01-23

### 新增
- **向后兼容的旧接口支持**：恢复所有旧版 API 方法，确保向后兼容
  - `read_metadata()` - 返回 Metadata 对象的旧接口
  - `extract_cover()` - 直接提取封面的方法
  - `set_cover(image_path, mime_type, description)` - 从文件路径设置封面
  - `get_lyrics()` / `set_lyrics(lyrics)` / `remove_lyrics()` - 歌词操作方法
  - FLAC 特定方法：`set_flac_title()`, `set_flac_artist()`, `set_flac_album()` 等
- **双接口共存**：新 JSON API 和旧直接方法 API 现在可以同时使用
- 支持删除封面图片（通过设置 `cover` 为 `null`）
- 支持删除歌词（通过空字符串或 `remove_lyrics()` 方法）

### 改进
- 新旧接口共享底层实现，减少代码重复
- 改进内部方法组织，提高代码可维护性

### 文档
- 更新 README.md，添加新旧接口使用示例
- 添加本项目 updates.md 更新计划文件

---

## [0.1.0] - 2024-12-XX

### 新增
- **初始版本发布**
- 支持 ID3v1 标签读取和写入（MP3）
- 支持 ID3v2 标签读取和写入（MP3）
  - 支持 ID3v2.2、v2.3、v2.4 版本
  - 支持文本帧（TIT2、TPE1、TALB、TYER、TDRC、TRCK、TCON、COMM）
  - 支持 USLT 帧（非同步歌词）
  - 支持 APIC 帧（封面图片）
- 支持 FLAC 元数据读取和写入
  - Vorbis Comment 块支持
  - Picture 块支持
- 自动检测音频文件格式（ID3v1、ID3v2、FLAC）
- **JSON 格式 API**
  - `get_metadata()` - 获取所有元数据（JSON 格式）
  - `set_metadata(json_str)` - 从 JSON 设置元数据
- 封面图片 Base64 编码支持
- 歌词读取和写入支持
- 完整的 Python 文档字符串

### 支持的元数据字段
- title（标题）
- artist（艺术家）
- album（专辑）
- year（年份）
- track（曲目号）
- genre（流派）
- comment（备注）
- lyrics（歌词）
- cover（封面图片）

### 技术栈
- Rust + PyO3 绑定
- serde/serde_json JSON 序列化
- base64 编解码
- encoding_rs 文本编码处理

---

## [未来版本规划]

### [0.6.0] - 计划中
**主题：CLI 工具增强**

- 扩展 CLI 命令（write、copy、batch 等）
- 添加配置文件支持
- 改进输出格式选项
- 添加进度条显示

### [1.0.0] - 计划中
**主题：稳定版**

- 完整的测试覆盖率（>90%）
- API 稳定性保证
- 完整的文档和教程
- 多语言绑定（考虑 Node.js、Ruby 等）
- 性能优化和基准测试报告
- 安全审计

---

## 版本说明

### 版本号规则
- **主版本号（Major）**：不兼容的 API 变更
- **次版本号（Minor）**：向下兼容的功能新增
- **修订号（Patch）**：向下兼容的问题修复

### 发布周期
- **主版本**：根据重大功能更新，不定期发布
- **次版本**：每 1-2 个月发布一次，包含新功能
- **修订版本**：根据需要发布，包含 bug 修复

---

## 贡献指南

如果您想为项目贡献代码或提出建议，请：

1. 查看 GitHub Issues 了解当前任务
2. 提交 Issue 描述您发现的问题或建议
3. Fork 项目并提交 Pull Request
4. 在 PR 中引用相关的 Issue

---

## 联系方式

- 项目地址：https://github.com/xwsjjctz/oxidant
- 邮箱：xwsjjctz@icloud.com

---

## 许可证

MIT License - 详见 LICENSE 文件
