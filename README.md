# Oxidant

一个高性能的音频元数据处理库，基于 Rust 编写，使用 PyO3 提供 Python 接口。支持 ID3 和 FLAC 格式音频文件的元数据读取。

## 功能特性

- ✅ 读取 ID3v1 标签（MP3 文件）
- ✅ 读取 ID3v2 标签（MP3 文件）
- ✅ 读取 FLAC 元数据（Vorbis Comment）
- ✅ 自动检测音频文件格式
- 🚀 高性能 Rust 实现
- 🐍 简单易用的 Python API

## 安装

### 环境要求

- Python 3.8+
- Rust 1.70+
- uv (推荐) 或 pip

### 使用 uv 安装

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 使用 uv 安装依赖
uv pip install -e .

# 或者使用 maturin 直接构建
uv run maturin develop
```

### 使用 pip 安装

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 安装 maturin
pip install maturin

# 构建并安装
maturin develop
```

## 快速开始

### 读取音频元数据

```python
import oxidant

# 创建 AudioFile 实例
audio_file = oxidant.AudioFile("path/to/your/audio.mp3")

# 读取元数据
metadata = audio_file.read_metadata()

# 访问元数据字段
print(f"标题: {metadata.title}")
print(f"艺术家: {metadata.artist}")
print(f"专辑: {metadata.album}")
print(f"年份: {metadata.year}")
print(f"曲目: {metadata.track}")
print(f"流派: {metadata.genre}")
print(f"备注: {metadata.comment}")
```

### 检测文件类型

```python
import oxidant

# 检测文件类型
file_type = oxidant.AudioFile.detect_file_type("path/to/audio.mp3")
print(f"文件类型: {file_type}")  # 输出: id3v2, id3v1, flac 或 unknown
```

### 提取封面图片

```python
import oxidant

# 读取元数据
audio_file = oxidant.AudioFile("path/to/audio.flac")
metadata = audio_file.read_metadata()

# 提取封面图片
cover = audio_file.extract_cover()
if cover:
    print(f"封面类型: {cover.mime_type}")
    print(f"封面尺寸: {cover.width}x{cover.height}")
    print(f"封面描述: {cover.description}")

    # 保存封面图片
    cover.save("cover.jpg")
    print("封面已保存为 cover.jpg")
else:
    print("未找到封面图片")
```

## API 文档

### AudioFile 类

#### 构造函数

```python
AudioFile(path: str) -> AudioFile
```

创建一个新的 AudioFile 实例。

**参数:**
- `path`: 音频文件路径

**返回:**
- `AudioFile` 实例

#### 属性

- `path`: 文件路径（只读）
- `file_type`: 文件类型（只读）

#### 方法

##### `read_metadata()`

读取音频文件的元数据。

**返回:**
- `Metadata` 对象

##### `extract_cover()`

提取音频文件的封面图片（仅支持 FLAC 格式）。

**返回:**
- `CoverArt` 对象或 `None`

##### `detect_file_type(path: str)` [静态方法]

检测音频文件的类型。

**参数:**
- `path`: 文件路径

**返回:**
- `str`: 文件类型（"id3v2", "id3v1", "flac" 或 "unknown"）

### Metadata 类

#### 属性

- `file_type`: 文件类型
- `version`: 版本信息
- `title`: 标题
- `artist`: 艺术家
- `album`: 专辑
- `year`: 年份
- `track`: 曲目号
- `genre`: 流派
- `comment`: 备注

所有属性都是可选的（`Option[str]`），可能为 `None`。

#### 方法

##### `to_dict()`

将元数据转换为字典。

**返回:**
- `dict`: 包含所有元数据的字典

## 支持的格式

### ID3 标签

- **ID3v1**: 基本的 MP3 标签格式
- **ID3v2**: 高级 MP3 标签格式（v2.2, v2.3, v2.4）

### FLAC

- **Vorbis Comment**: FLAC 的元数据格式
- **Picture Block**: 封面图片

## 开发

### 环境设置

```bash
# 克隆仓库
git clone https://github.com/yourusername/oxidant.git
cd oxidant

# 设置 Python 版本
uv python pin 3.12.9

# 安装开发依赖
uv pip install maturin

# 构建开发版本
uv run maturin develop
```

### 运行测试

```bash
# 运行基本测试
uv run python test_oxidant.py

# 运行完整测试
uv run python test.py
```

### 项目结构

```
oxidant/
├── src/
│   ├── lib.rs              # PyO3 绑定入口
│   ├── id3/                # ID3 标签处理
│   │   ├── mod.rs
│   │   ├── v1.rs           # ID3v1 实现
│   │   ├── v2.rs           # ID3v2 实现
│   │   └── frames.rs       # 帧类型定义
│   ├── flac/               # FLAC 元数据处理
│   │   ├── mod.rs
│   │   ├── metadata.rs     # 元数据块
│   │   ├── vorbis.rs       # Vorbis Comment
│   │   └── picture.rs      # 图片块
│   └── utils/              # 工具函数
│       ├── mod.rs
│       ├── encoding.rs     # 编码转换
│       └── io.rs           # I/O 工具
├── Cargo.toml              # Rust 项目配置
├── pyproject.toml          # Python 项目配置
└── README.md
```

## 性能

Oxidant 使用 Rust 实现，提供了接近原生 C 的性能：

- **快速解析**: 手动解析字节流，避免不必要的内存拷贝
- **低内存占用**: 使用零拷贝技术读取数据
- **并发安全**: Rust 的所有权系统确保线程安全

## 依赖项

### Rust 依赖

- `pyo3`: Python 绑定
- `encoding_rs`: 文本编码处理

### Python 依赖

- 无额外依赖

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 联系方式

- 作者: xwsjjctz
- 邮箱: xwsjjctz@icloud.com
- 项目主页: https://github.com/xwsjjctz/oxidant

## 致谢

- [PyO3](https://github.com/PyO3/pyo3) - Rust 的 Python 绑定
- [Maturin](https://github.com/PyO3/maturin) - Rust 扩展构建工具
- [ID3 规范](http://id3.org/)
- [FLAC 规范](https://xiph.org/flac/format.html)