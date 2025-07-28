# PigSudoku - Dioxus Sudoku Game

A Sudoku game built with the Dioxus framework in Rust, by vibe coding.

## Todo List

### Phase 1: Minimal App Setup
- [x] 1. Create a minimal app with Dioxus (Hello World)
  - Set up basic Dioxus project structure
  - Implement simple "Hello World" component
  - Configure Cargo.toml with Dioxus dependencies

### Phase 2: Sudoku Game Implementation
- [x] 1. allow user to input via keyboard
- [x] 2. random generate a new game
- [x] 3. use different color for given number and number for user input
- [x] 4. add a solve helper button, per click, fill one empty cell with correct number
- [x] make different level of new game, easy, medium, hard, based on the number of given number, let the player chose when gen new game
- [x] ensure the puzzle has unique solution
- [x] make all the cell clickable, when one cell with number clicked, make all cells with the same number clicked
- [x] when the puzzle is solved, give more vivid congratulation message may be with animation
- [x] when an empty cell clicked, pop up a option cube with number 1-9, when a number clicked, fill the cell with the number, and close the cube. During the pop up of the cube, keyboard input shall be still acceptable.
- [x] It seems when the player type or chose the wrong anwser, the game board will block what the player typed. Don't block, let the player keep inputing until there are obvious errors, when the error happened give some alert mark
- [x] In the right position of the puzzle field, make a log field, show the user's activities, like manually input or hint input of position and number. With the log, the player shall be able to revert or redo every step. We shall have buttons of revert and redo, also treat keyboard input left arrow and up arrow as revert, and right arrow and down arrow as redo.
- [x] Add a new feature: note, allow the player to fill all the data 1-9, the noted number shall be filled in smaller size, like all the 9 numbers together can be filled in one cell.
  - in the popup box, player can right click to note a number, after right click, the corresponding popup cell change it's color to yellow, and the number filled in the cell change to smaller size, the popup box don't close instantly, it only closes when the player click the "note done" button in the popup, or left click a number to show it's already filled, or click Enter key, which shall act same as left click the "note done" button.
  - the noted numbers shall not be considered as the answer, when the player click the "solve" button, the noted numbers shall be ignored.
  - when the user fill the cell with a number, the note shall be visiblely cleared, but data still exists. when the user clear the filled cell, we shall be able to see it again.
- instead of note and fill in the popup box, we can make the box be solid in the right part of the game zone, below the log board. then the box can have a note button, when it's toggled, left click on the number in the box is to note, when it's toggled off, left click on the number in the box is to fill in the number, the toggle status shall be visible via color
- give user a progress bar, when it takes a long time to generate puzzle
- when give hint number, also give text format reason
  - read https://www.sudokudragon.com/sudokututorials.htm

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Cargo
- Web browser for testing

### Development

```bash
# Run the app in development mode
cargo run
```

## Project Structure

```
pigsudoku/
├── src/
│   └── main.rs          # Entry point and Sudoku game implementation
├── Cargo.toml          # Dependencies and project configuration
├── Cargo.lock          # Dependency lock file
└── README.md           # This file
```

## Features

- ✅ **Interactive 9x9 Sudoku Grid**: Click cells to select and input numbers
- ✅ **Keyboard Input**: Use number keys (1-9) to fill cells, arrow keys to navigate
- ✅ **Random Puzzle Generation**: Each new game generates a unique, solvable puzzle
- ✅ **Visual Distinction**: Different colors for given numbers vs user input
- ✅ **Real-time Validation**: Prevents invalid moves according to Sudoku rules
- ✅ **Hint System**: Solve helper button fills one correct cell per click
- ✅ **Game Controls**: Clear cell, new game, and hint functionality
- ✅ **Win Detection**: Congratulations message when puzzle is solved
- ✅ **Responsive UI**: Clean, modern interface with proper styling
- ✅ **Keyboard Navigation**: Full keyboard support for seamless gameplay

## Technologies

- **Dioxus 0.6**: Modern Rust GUI framework
- **Rust**: Systems programming language
- **Desktop**: Current target platform

## Roadmap

### Future Features
- **Web Compatibility**: Enable web variant of Dioxus
  - Test web compilation and rendering
  - Set up web-specific configurations
  - Verify cross-browser compatibility
  - WebAssembly optimization
- **Mobile Support**: Extend to mobile platforms
- **Advanced Game Features**:
  - Multiple difficulty levels
  - Hint system
  - Timer and scoring
  - Save/load game state
  - Dark/light theme toggle

---

# PigSudoku - Dioxus 数独游戏

一个使用 Rust 的 Dioxus 框架构建的数独游戏，通过氛围编程开发。

## 待办事项列表

### 第一阶段：最小应用设置
- [x] 1. 使用 Dioxus 创建最小应用（Hello World）
  - 设置基本的 Dioxus 项目结构
  - 实现简单的 "Hello World" 组件
  - 配置 Cargo.toml 的 Dioxus 依赖

### 第二阶段：数独游戏实现
- [x] 1. 允许用户通过键盘输入
- [x] 2. 随机生成新游戏
- [x] 3. 为给定数字和用户输入数字使用不同颜色
- [x] 4. 添加求解助手按钮，每次点击填充一个空单元格的正确数字
- [x] 制作不同难度的新游戏：简单、中等、困难，基于给定数字的数量，让玩家在生成新游戏时选择
- [x] 确保谜题有唯一解
- [x] 使所有单元格可点击，当点击一个有数字的单元格时，高亮显示所有相同数字的单元格
- [x] 当谜题解决时，给出更生动的祝贺信息，可能带有动画
- [x] 当点击空单元格时，弹出一个包含数字1-9的选项立方体，当点击数字时，用该数字填充单元格并关闭立方体。在立方体弹出期间，键盘输入仍应可接受。
- [x] 似乎当玩家输入或选择错误答案时，游戏板会阻止玩家输入的内容。不要阻止，让玩家继续输入直到出现明显错误，当错误发生时给出一些警告标记
- [x] 在谜题字段的右侧位置，制作一个日志字段，显示用户的活动，如手动输入或提示输入的位置和数字。通过日志，玩家应该能够撤销或重做每一步。我们应该有撤销和重做按钮，同时将键盘输入的左箭头和上箭头视为撤销，右箭头和下箭头视为重做。
- 给用户一个进度条，当生成谜题需要很长时间时
- 当给出提示数字时，也给出文本格式的原因
  - 阅读 https://www.sudokudragon.com/sudokututorials.htm

## 开始使用

### 先决条件
- Rust（最新稳定版本）
- Cargo
- 用于测试的网页浏览器

### 开发

```bash
# 在开发模式下运行应用
cargo run
```

## 项目结构

```
pigsudoku/
├── src/
│   └── main.rs          # 入口点和数独游戏实现
├── Cargo.toml          # 依赖和项目配置
├── Cargo.lock          # 依赖锁定文件
└── README.md           # 此文件
```

## 功能特性

- ✅ **交互式 9x9 数独网格**：点击单元格选择并输入数字
- ✅ **键盘输入**：使用数字键（1-9）填充单元格，箭头键导航
- ✅ **随机谜题生成**：每个新游戏生成独特的可解谜题
- ✅ **视觉区分**：给定数字与用户输入使用不同颜色
- ✅ **实时验证**：根据数独规则防止无效移动
- ✅ **提示系统**：求解助手按钮每次点击填充一个正确单元格
- ✅ **游戏控制**：清除单元格、新游戏和提示功能
- ✅ **胜利检测**：谜题解决时显示祝贺信息
- ✅ **响应式界面**：干净、现代的界面和适当的样式
- ✅ **键盘导航**：完整的键盘支持，实现无缝游戏体验

## 技术栈

- **Dioxus 0.6**：现代 Rust GUI 框架
- **Rust**：系统编程语言
- **桌面**：当前目标平台

## 路线图

### 未来功能
- **Web 兼容性**：启用 Dioxus 的 Web 变体
  - 测试 Web 编译和渲染
  - 设置 Web 特定配置
  - 验证跨浏览器兼容性
  - WebAssembly 优化
- **移动支持**：扩展到移动平台
- **高级游戏功能**：
  - 多个难度级别
  - 提示系统
  - 计时器和评分
  - 保存/加载游戏状态
  - 深色/浅色主题切换