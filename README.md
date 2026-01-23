# Oxidant

一个高性能的音频元数据处理库，基于 Rust 编写，使用 PyO3 提供 Python 接口。支持多种音频格式的元数据读写。

## 功能特性

- ✅ 读取 ID3v1 标签（MP3 文件）
- ✅ 读取 ID3v2 标签（MP3 文件）
- ✅ 读取 FLAC 元数据（Vorbis Comment）
- ✅ 读取 OGG Vorbis 元数据（Vorbis Comment）
- ✅ 读取 OPUS 元数据（OpusTags/Vorbis Comment）
- ✅ 读取 MP4/M4A 元数据（iTunes 风格 atoms）
- ✅ 读取 APE 元数据（APE Tags）
- ✅ 写入 ID3v1 标签（MP3 文件）
- ✅ 写入 ID3v2 标签（MP3 文件）
- ✅ 写入 FLAC 元数据（Vorbis Comment）
- ✅ 写入 OGG Vorbis 元数据（Vorbis Comment）
- ✅ 写入 OPUS 元数据（OpusTags/Vorbis Comment）
- ✅ 读取和写入封面图片（ID3v2 APIC、FLAC Picture）
- ✅ 读取和写入歌词（ID3v2 USLT、FLAC LYRICS、OGG LYRICS、OPUS LYRICS）
- ✅ 自动检测音频文件格式
- ✅ 批量处理多个音频文件
- 🚀 高性能 Rust 实现
- 🐍 简单易用的 Python API
- 📦 JSON 格式的元数据交换
- 🔧 统一的元数据字段映射系统
- 🛠️ 独立的 CLI 命令行工具
- 📚 纯 Rust 库支持（不依赖 Python）

## 支持的格式

### 完整支持（读写）

| 格式 | 扩展名 | 元数据格式 | 说明 |
|------|--------|-----------|------|
| **MP3** | .mp3 | ID3v1, ID3v2 | 完整的读写支持 |
| **FLAC** | .flac | Vorbis Comment, Picture | 完整的读写支持 |
| **OGG Vorbis** | .ogg | Vorbis Comment | 完整的读写支持 |
| **OPUS** | .opus | OpusTags (Vorbis Comment) | 完整的读写支持 |

### 只读支持

| 格式 | 扩展名 | 元数据格式 | 说明 |
|------|--------|-----------|------|
| **MP4/M4A** | .mp4, .m4a | iTunes atoms | 读取支持，写入待实现 |
| **APE** | .ape | APE Tags | 读取支持，写入待实现 |

## 安装

### 环境要求

- Python 3.8+ （仅 Python 模块）
- Rust 1.70+
- uv (推荐) 或 pip （仅 Python 模块）
- cargo （Rust 库和 CLI）

### 三种使用方式

Oxidant 支持三种使用方式：

#### 1. Python 模块

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 使用 uv 安装依赖
uv pip install -e .

# 或者使用 maturin 直接构建
uv run maturin develop
```

#### 2. Rust 库

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 构建为 Rust 库
cargo build --lib

# 在其他 Rust 项目中使用
# 在 Cargo.toml 中添加：
# [dependencies]
# oxidant = { path = "/path/to/oxidant" }
```

#### 3. CLI 工具

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 构建 CLI 工具
cargo build --release

# CLI 二进制文件位置
# 开发版本: target/debug/oxidant
# 发布版本: target/release/oxidant

# 使用示例
./target/release/oxidant --help
./target/release/oxidant detect song.mp3
./target/release/oxidant read song.mp3
```

### 使用 uv 安装（仅 Python 模块）

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 使用 uv 安装依赖
uv pip install -e .

# 或者使用 maturin 直接构建
uv run maturin develop
```

**注意**：如果不使用 uv，也可以使用 pip：
```bash
pip install maturin
maturin develop
```

## 快速开始

### 读取音频元数据

```python
import oxidant
import json

# 创建 AudioFile 实例
audio_file = oxidant.AudioFile("path/to/your/audio.mp3")

# 获取元数据（JSON 格式）
metadata_json = audio_file.get_metadata()
metadata = json.loads(metadata_json)

# 访问元数据字段
print(f"文件类型: {metadata['file_type']}")
print(f"标题: {metadata.get('title')}")
print(f"艺术家: {metadata.get('artist')}")
print(f"专辑: {metadata.get('album')}")
print(f"年份: {metadata.get('year')}")
print(f"曲目: {metadata.get('track')}")
print(f"流派: {metadata.get('genre')}")
print(f"备注: {metadata.get('comment')}")
print(f"歌词: {metadata.get('lyrics')}")

# 获取封面图片（如果存在）
if 'cover' in metadata:
    cover = metadata['cover']
    print(f"封面类型: {cover['mime_type']}")
    print(f"封面尺寸: {cover['width']}x{cover['height']}")
```

### 检测文件类型

```python
import oxidant

# 创建 AudioFile 实例（自动检测文件类型）
audio_file = oxidant.AudioFile("path/to/audio.mp3")
print(f"文件类型: {audio_file.file_type}")  # 输出: id3v2, id3v1, flac 或 unknown
```

### 写入音频元数据

```python
import oxidant
import json

# 创建 AudioFile 实例
audio_file = oxidant.AudioFile("path/to/audio.mp3")

# 准备要写入的元数据
new_metadata = {
    "title": "新标题",
    "artist": "新艺术家",
    "album": "新专辑",
    "year": "2024",
    "track": "1",
    "genre": "Pop",
    "comment": "这是备注",
    "lyrics": "这里是歌词内容..."
}

# 写入元数据
audio_file.set_metadata(json.dumps(new_metadata))
print("元数据已更新")
```

### 更新封面图片

```python
import oxidant
import json
import base64

# 读取图片文件
with open("cover.jpg", "rb") as f:
    cover_data = base64.b64encode(f.read()).decode('utf-8')

# 创建 AudioFile 实例
audio_file = oxidant.AudioFile("path/to/audio.flac")

# 准备包含封面的元数据
metadata_with_cover = {
    "cover": {
        "mime_type": "image/jpeg",
        "width": 1000,
        "height": 1000,
        "depth": 24,
        "description": "封面图片",
        "data": cover_data
    }
}

# 写入元数据和封面
audio_file.set_metadata(json.dumps(metadata_with_cover))
print("封面已更新")
```

### 删除封面图片

```python
import oxidant
import json

# 创建 AudioFile 实例
audio_file = oxidant.AudioFile("path/to/audio.mp3")

# 设置 cover 为 null 以删除封面
metadata_without_cover = {
    "cover": None
}

audio_file.set_metadata(json.dumps(metadata_without_cover))
print("封面已删除")
```

## CLI 工具

Oxidant 提供了命令行工具 `oxidant`，可以快速读取和检测音频文件的元数据。

### 安装 CLI

```bash
# 构建并安装 Python 模块和 CLI
uv run maturin develop

# 或使用 cargo 直接构建 CLI
cargo build --release

# CLI 二进制文件位置
# 开发版本: target/debug/oxidant
# 发布版本: target/release/oxidant
```

### 使用示例

#### 查看帮助

```bash
oxidant --help
```

#### 检测文件格式

```bash
# 检测单个文件
oxidant detect song.mp3

# 检测多个文件
oxidant detect song.mp3 album.flac track.ogg
```

#### 读取元数据

```bash
# 读取并显示元数据（JSON 格式）
oxidant read song.mp3

# 安静模式（仅输出 JSON）
oxidant read song.mp3 --quiet

# 输出到文件
oxidant read song.mp3 --output metadata.json
```

### CLI 命令

| 命令 | 说明 |
|------|------|
| `read` | 读取音频文件的元数据 |
| `detect` | 检测音频文件的格式 |
| `--help` | 显示帮助信息 |
| `--version` | 显示版本信息 |

### CLI 选项

| 选项 | 简写 | 说明 |
|------|------|------|
| `--format` | `-f` | 输出格式（pretty/json） |
| `--quiet` | `-q` | 安静模式，仅输出结果 |

## API 文档

### AudioFile 类

#### 构造函数

```python
AudioFile(path: str) -> AudioFile
```

创建一个新的 AudioFile 实例，自动检测文件类型。

**参数:**
- `path`: 音频文件路径

**返回:**
- `AudioFile` 实例

#### 属性

- `path` (str): 文件路径（只读）
- `file_type` (str): 文件类型（只读）
  - `"id3v2"`: ID3v2 标签（MP3）
  - `"id3v1"`: ID3v1 标签（MP3）
  - `"flac"`: FLAC 格式
  - `"unknown"`: 未知格式

#### 方法

##### `get_metadata() -> str`

读取音频文件的所有元数据，包括封面图片。

**返回:**
- `str`: JSON 格式的元数据字符串

**JSON 结构:**
```json
{
  "file_type": "ID3v2",
  "version": "3.0",
  "title": "歌曲标题",
  "artist": "艺术家",
  "album": "专辑名称",
  "year": "2024",
  "track": "1",
  "genre": "Pop",
  "comment": "备注",
  "lyrics": "歌词内容...",
  "cover": {
    "mime_type": "image/jpeg",
    "width": 1000,
    "height": 1000,
    "depth": 24,
    "description": "封面描述",
    "data": "base64编码的图片数据"
  }
}
```

**注意:**
- 所有字段都是可选的，不存在的字段不会出现在 JSON 中
- `cover` 字段仅当文件包含封面图片时才存在
- 图片数据以 Base64 编码的字符串形式存储

##### `set_metadata(json_str: str) -> None`

根据 JSON 字符串更新音频文件的元数据。

**参数:**
- `json_str`: JSON 格式的元数据字符串

**更新行为:**
- 只更新 JSON 中存在的字段
- 未包含的字段保持不变
- 设置字段为空字符串（`""`）会删除该字段
- 设置 `cover` 为 `null` 会删除封面图片
- 不包含 `cover` 字段时，保持原有封面不变

**示例:**
```python
# 只更新标题和艺术家，其他字段保持不变
audio_file.set_metadata('{"title": "新标题", "artist": "新艺术家"}')

# 删除歌词
audio_file.set_metadata('{"lyrics": ""}')

# 删除封面
audio_file.set_metadata('{"cover": null}')
```

**异常:**
- `PyValueError`: JSON 格式无效或文件类型不支持
- `PyIOError`: 文件读写错误

## 支持的格式

### ID3 标签（MP3）

**ID3v1**
- 固定 128 字节标签
- 位于文件末尾
- 支持字段：title, artist, album, year, comment, track, genre

**ID3v2**
- 可变长度标签
- 位于文件开头
- 支持 ID3v2.2、v2.3、v2.4 版本
- 支持字段：title, artist, album, year, track, genre, comment, lyrics
- 支持封面图片（APIC 帧）

### FLAC

**Vorbis Comment**
- 标准元数据块
- 支持字段：TITLE, ARTIST, ALBUM, DATE, TRACKNUMBER, GENRE, COMMENT, LYRICS

**Picture Block**
- 封面图片块
- 支持多种图片格式（JPEG, PNG 等）

### OGG Vorbis

**Vorbis Comment**
- 使用与 FLAC 相同的 Vorbis Comment 格式
- 位于第二个 OGG 页面（Comment Header）
- 支持字段：TITLE, ARTIST, ALBUM, DATE, TRACKNUMBER, GENRE, COMMENT, LYRICS

**OGG 容器**
- 使用 OGG 页面结构封装
- 自动识别 OGG 签名

### 其他格式（基础框架已实现）

**OPUS**
- 基础框架已完成（`src/opus/mod.rs`）
- 使用 OGG 容器 + Vorbis Comment
- 待实现完整读写功能

**MP4/M4A**
- 基础框架已完成（`src/mp4/mod.rs`）
- 使用 iTunes 风格原子（atom）结构
- 支持字段：©nam, ©ART, ©alb, ©day, trkn, ©gen, ©cmt, ©lyr, covr
- 待实现完整读写功能

**APE**
- 基础框架已完成（`src/ape/mod.rs`）
- 使用 APE 标签格式
- 支持字段：Title, Artist, Album, Year, Track, Genre, Comment, Lyrics
- 待实现完整读写功能

### 统一字段映射

项目实现了统一的元数据字段映射系统（`src/field_mapping.rs`），支持：
- 标准化字段名称（title, artist, album, year, track, genre, comment, lyrics, cover）
- 各格式特定字段的自动转换
- 格式特定的值处理（如年份规范化、曲目号解析）

## 元数据字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `file_type` | string | 文件类型（只读） |
| `version` | string | 标签版本（只读） |
| `title` | string? | 歌曲标题 |
| `artist` | string? | 艺术家/歌手 |
| `album` | string? | 专辑名称 |
| `year` | string? | 发行年份 |
| `track` | string? | 曲目编号 |
| `genre` | string? | 音乐流派 |
| `comment` | string? | 备注信息 |
| `lyrics` | string? | 歌词文本 |
| `cover` | object? | 封面图片对象 |

**封面图片对象结构:**
```json
{
  "mime_type": "image/jpeg",  // MIME 类型
  "width": 1000,              // 宽度（像素）
  "height": 1000,             // 高度（像素）
  "depth": 24,                // 色深
  "description": "",          // 描述文字
  "data": "base64..."         // Base64 编码的图片数据
}
```

## 高级用法

### 批量处理音频文件

Oxidant 0.4.0+ 提供了专用的 `BatchProcessor` 类用于批量处理音频文件：

```python
import oxidant
import json

# 创建批量处理器
processor = oxidant.BatchProcessor()
processor.show_progress = True  # 显示处理进度

# 方式一：批量读取元数据
file_paths = ["song1.mp3", "song2.flac", "song3.ogg"]
metadata_list = processor.read_metadata_batch(file_paths)

for metadata_json in metadata_list:
    metadata = json.loads(metadata_json)
    print(f"标题: {metadata.get('title')}, 艺术家: {metadata.get('artist')}")

# 方式二：批量写入元数据
updates = [
    ("song1.mp3", json.dumps({"artist": "Artist A"})),
    ("song2.flac", json.dumps({"artist": "Artist B"})),
    ("song3.ogg", json.dumps({"artist": "Artist C"})),
]
results = processor.write_metadata_batch(updates)

for result in results:
    if result.success:
        print(f"✓ {result.file_path}")
    else:
        print(f"✗ {result.file_path}: {result.error_message}")

# 方式三：处理整个目录
# 读取目录中所有 FLAC 文件的元数据
flac_metadata = processor.process_directory(
    directory="./music",
    pattern="*.flac",
    operation="read"
)

# 批量更新目录中所有 MP3 文件的元数据
mp3_results = processor.process_directory(
    directory="./music",
    pattern="*.mp3",
    operation="write",
    metadata_json=json.dumps({"artist": "统一艺术家名称", "year": "2024"})
)

# 方式四：复制元数据
processor.copy_metadata("source.mp3", "target.flac")
print("元数据已复制")
```

### BatchProcessor 类

#### 属性

- `show_progress` (bool): 是否显示处理进度信息（默认: True）

#### 方法

##### `read_metadata_batch(file_paths: List[str]) -> List[str]`

批量读取多个音频文件的元数据。

**参数:**
- `file_paths`: 文件路径列表

**返回:**
- `List[str]`: JSON 格式的元数据字符串列表

##### `write_metadata_batch(updates: List[Tuple[str, str]]) -> List[BatchResult]`

批量写入元数据到多个音频文件。

**参数:**
- `updates`: 元组列表，每个元组包含 (file_path, metadata_json)

**返回:**
- `List[BatchResult]`: 处理结果列表

##### `process_directory(directory: str, pattern: str, operation: str, metadata_json: Optional[str] = None)`

处理目录中匹配指定模式的所有文件。

**参数:**
- `directory`: 目录路径
- `pattern`: 文件模式（如 "*.mp3", "*.flac"）
- `operation`: 操作类型 ("read" 或 "write")
- `metadata_json`: 要写入的元数据 JSON（write 操作时必需）

**返回:**
- 读取操作返回元数据列表，写入操作返回 `BatchResult` 列表

##### `copy_metadata(source_path: str, target_path: str)`

将源文件的元数据复制到目标文件。

#### BatchResult 类

批量处理的结果对象。

**属性:**
- `file_path` (str): 文件路径
- `success` (bool): 是否成功
- `error_message` (str?): 错误信息（如果失败）

### 旧式批量处理（兼容代码）

```python
import oxidant
import json
import os
from pathlib import Path

def process_audio_files(directory):
    """批量处理目录中的所有音频文件"""
    for audio_file in Path(directory).glob("*.mp3"):
        try:
            audio = oxidant.AudioFile(str(audio_file))
            metadata = json.loads(audio.get_metadata())

            print(f"处理: {audio_file.name}")
            print(f"  标题: {metadata.get('title')}")
            print(f"  艺术家: {metadata.get('artist')}")

            # 批量更新某个字段
            new_metadata = {"artist": "统一艺术家名称"}
            audio.set_metadata(json.dumps(new_metadata))

        except Exception as e:
            print(f"错误: {audio_file.name} - {e}")

process_audio_files("./music")
```

### 保存封面图片

```python
import oxidant
import json
import base64

audio_file = oxidant.AudioFile("song.flac")
metadata = json.loads(audio_file.get_metadata())

if 'cover' in metadata:
    cover = metadata['cover']

    # 解码 Base64 数据
    image_data = base64.b64decode(cover['data'])

    # 根据类型确定扩展名
    ext_map = {
        "image/jpeg": ".jpg",
        "image/png": ".png",
        "image/gif": ".gif"
    }
    ext = ext_map.get(cover['mime_type'], ".jpg")

    # 保存图片
    output_file = f"cover{ext}"
    with open(output_file, "wb") as f:
        f.write(image_data)

    print(f"封面已保存到: {output_file}")
else:
    print("文件没有封面")
```

### 元数据备份与恢复

```python
import oxidant
import json

# 备份元数据
audio = oxidant.AudioFile("song.mp3")
metadata_backup = audio.get_metadata()

with open("metadata_backup.json", "w") as f:
    f.write(metadata_backup)

# 恢复元数据
with open("metadata_backup.json", "r") as f:
    backup_data = f.read()

audio.set_metadata(backup_data)
print("元数据已恢复")
```

## 开发

### 环境设置

```bash
# 克隆仓库
git clone https://github.com/xwsjjctz/oxidant.git
cd oxidant

# 设置 Python 版本
uv python pin 3.12.9

# 安装开发依赖
uv pip install maturin

# 构建开发版本
uv run maturin develop

# 或使用 pip 安装
pip install maturin
maturin develop
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
│   ├── ogg/                # OGG 容器处理
│   │   ├── mod.rs
│   │   ├── page.rs         # OGG 页面结构
│   │   └── vorbis.rs       # OGG Vorbis Comment
│   ├── opus/               # OPUS 格式处理
│   │   └── mod.rs          # OPUS 实现
│   ├── mp4/                # MP4/M4A 格式处理
│   │   └── mod.rs          # iTunes atoms 实现
│   ├── ape/                # APE 格式处理
│   │   └── mod.rs          # APE Tags 实现
│   └── utils/              # 工具函数
│       ├── mod.rs
│       ├── encoding.rs     # 编码转换
│       └── io.rs           # I/O 工具
├── Cargo.toml              # Rust 项目配置
└── README.md
```

## 性能

Oxidant 使用 Rust 实现，提供了接近原生 C 的性能：

- **快速解析**: 手动解析字节流，避免不必要的内存拷贝
- **低内存占用**: 使用零拷贝技术读取数据
- **并发安全**: Rust 的所有权系统确保线程安全
- **高效编码**: 支持 UTF-8/UTF-16/ISO-8859-1 等多种编码自动识别

## 依赖项

### Rust 依赖

- `pyo3` (0.27.2): Python 绑定
- `encoding_rs` (0.8): 文本编码处理
- `serde` (1.0): 序列化/反序列化
- `serde_json` (1.0): JSON 支持
- `base64` (0.22): Base64 编解码

### Python 依赖

- 无额外运行时依赖

## 常见问题

### Q: 为什么使用 JSON 格式交换元数据？

A: JSON 格式提供了以下优势：
- 跨语言兼容性好
- 支持复杂嵌套结构（如封面图片对象）
- 便于调试和日志记录
- 易于与数据库、API 集成

### Q: 支持哪些音频格式？

A: 目前支持：
- **完整支持（读写）**:
  - **MP3**（ID3v1 和 ID3v2 标签）
  - **FLAC**（Vorbis Comment、Picture）
  - **OGG Vorbis**（Vorbis Comment）
  - **OPUS**（OpusTags/Vorbis Comment）
- **只读支持**:
  - **MP4/M4A**（iTunes 风格 atoms）
  - **APE**（APE Tags）

### Q: 封面图片数据为什么使用 Base64 编码？

A: Base64 编码可以将二进制数据安全地嵌入 JSON 文本中，便于传输和存储。如果需要直接处理二进制数据，可以使用 Python 的 `base64` 模块解码。

### Q: 修改元数据会重新编码音频吗？

A: 不会。Oxidant 只修改元数据部分，不会重新编码音频数据，因此速度极快且不会损失音质。

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 联系方式

- 邮箱: xwsjjctz@icloud.com

## 致谢

- [PyO3](https://github.com/PyO3/pyo3) - Rust 的 Python 绑定
- [Maturin](https://github.com/PyO3/maturin) - Rust 扩展构建工具
- [ID3 规范](http://id3.org/) - ID3 标签标准
- [FLAC 规范](https://xiph.org/flac/format.html) - FLAC 格式标准