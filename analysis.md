# 音频元数据处理库 - 需求分析

## 项目概述
开发一个基于 Rust 的 Python 库，用于处理 ID3 和 FLAC 编码音频的元数据信息，使用 Maturin 和 PyO3 实现。

## 技术架构

### 核心技术栈
- **实现语言**: Rust
- **Python 绑定**: PyO3
- **构建工具**: Maturin
- **目标语言**: Python

### 设计理念
- 手动解析音频文件的字节流
- 遵循 ID3 和 FLAC 的官方元数据标准规范
- 提供高性能、易用的 Python API

## 功能模块

### 1. ID3 元数据处理
- **支持的版本**: ID3v1.0、v1.1、v2.2、v2.3、v2.4
- **核心功能**:
  - 读取 ID3 标签信息（标题、艺术家、专辑、年份、曲目等）
  - 修改和写入 ID3 标签
  - 处理不同帧类型（TIT2、TPE1、TALB、TYER 等）
  - 支持封面图片（APIC 帧）

### 2. FLAC 元数据处理
- **核心功能**:
  - 读取 FLAC VORBIS_COMMENT 元数据
  - 修改和写入元数据
  - 处理 METADATA_BLOCK 结构
  - 支持封面图片（PICTURE block）

## 技术挑战

### ID3 格式复杂性
1. **多版本兼容**: ID3v2 各版本帧结构差异较大
2. **编码处理**: 需要支持 UTF-8、UTF-16、UTF-16BE、ISO-8859-1
3. **帧同步**: ID3v2 使用同步安全编码
4. **扩展头部**: 部分标签包含扩展头部

### FLAC 格式特点
1. **元数据块链**: FLAC 使用多个 METADATA_BLOCK 链式结构
2. **VORBIS_COMMENT**: 类似 Ogg Vorbis 的注释格式
3. **大小端**: FLAC 使用大端序（Big Endian）

## 实现建议

### 分阶段开发路线

#### 阶段 1: 基础框架
- 搭建 Maturin + PyO3 项目结构
- 实现基本的文件读取和字节流解析工具
- 设计 Python API 接口

#### 阶段 2: ID3v2.3 实现（最常用版本）
- 解析 ID3v2.3 标签头部
- 实现常用帧的读取（TIT2、TPE1、TALB、TYER、TRCK）
- 处理文本编码转换
- 实现标签写入功能

#### 阶段 3: ID3 其他版本
- 扩展支持 ID3v2.2、v2.4
- 实现 ID3v1 读取和写入

#### 阶段 4: FLAC 支持
- 解析 FLAC 文件结构
- 实现 VORBIS_COMMENT 读写
- 处理 PICTURE block

#### 阶段 5: 高级功能
- 封面图片处理
- 批量处理优化
- 错误处理和日志

### 关键技术点

1. **字节流解析**
   - 使用 Rust 的 `std::io::Read` trait
   - 处理变长整数编码
   - 注意大小端序转换

2. **编码处理**
   - 使用 `encoding_rs` 或类似 crate 处理文本编码
   - 正确转换各种字符集

3. **Python API 设计**
   - 提供简洁的类和方法接口
   - 支持 Python 的 `with` 语句上下文管理
   - 良好的错误处理和类型提示

4. **性能优化**
   - 避免不必要的内存拷贝
   - 使用零拷贝技术读取数据
   - 延迟加载大型帧（如封面图片）

## 项目结构建议

```
oxidant/
├── Cargo.toml
├── pyproject.toml
├── src/
│   ├── lib.rs              # PyO3 绑定入口
│   ├── id3/
│   │   ├── mod.rs
│   │   ├── v1.rs           # ID3v1 实现
│   │   ├── v2.rs           # ID3v2 通用实现
│   │   ├── v2_3.rs         # ID3v2.3 特定实现
│   │   ├── v2_4.rs         # ID3v2.4 特定实现
│   │   └── frames.rs       # 帧类型定义
│   ├── flac/
│   │   ├── mod.rs
│   │   ├── metadata.rs     # FLAC 元数据块
│   │   └── vorbis.rs       # VORBIS_COMMENT 处理
│   └── utils/
│       ├── mod.rs
│       ├── encoding.rs     # 编码转换工具
│       └── io.rs           # 字节流读取工具
└── tests/
    ├── fixtures/           # 测试音频文件
    └── test_*.py          # Python 测试
```

## 测试策略

1. **单元测试**: Rust 层的解析逻辑测试
2. **集成测试**: Python API 的功能测试
3. **真实文件测试**: 使用各种编码和版本的音频文件
4. **边界测试**: 空标签、损坏文件、超大标签等

## 参考资料

- [ID3v2.4 规范](http://id3.org/id3v2.4.0-frames)
- [ID3v2.3 规范](http://id3.org/id3v2.3.0)
- [FLAC 格式规范](https://xiph.org/flac/format.html)
- [PyO3 官方文档](https://pyo3.rs/)
- [Maturin 官方文档](https://www.maturin.rs/)

## 潜在风险

1. **规范理解偏差**: 需要仔细阅读官方规范文档
2. **兼容性问题**: 不同软件生成的标签可能有细微差异
3. **性能瓶颈**: 大文件处理需要优化内存使用
4. **测试覆盖**: 需要准备足够的测试用例

## 总结

这是一个具有挑战性但很有价值的项目。手动实现音频元数据解析不仅能深入理解音频格式，还能提供高性能的 Python 库。建议采用渐进式开发策略，先实现最常用的 ID3v2.3，再逐步扩展功能。