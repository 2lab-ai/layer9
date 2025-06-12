# Layer9 Todo App Demo Guide

## Running the Todo App

1. Build the app (if not already built):
   ```bash
   wasm-pack build --target web --out-dir pkg
   ```

2. Start the server:
   ```bash
   python3 -m http.server 8082
   ```

3. Open in browser: http://localhost:8082

## Demo Scenarios for Screenshots/GIFs

### 1. Empty State
- Show the clean, empty todo app
- Highlight: "No todos yet!" message
- Beautiful gradient background

### 2. Adding Todos
- Type "Build a Rust web framework" and click Add
- Type "Create beautiful examples" and click Add
- Type "Write comprehensive documentation" and click Add
- Shows the smooth addition of items

### 3. Completing Todos
- Click checkbox next to "Build a Rust web framework"
- Shows strikethrough effect and color change
- Updates the Active/Completed counts

### 4. Using Filters
- Click "Active" filter - shows only uncompleted todos
- Click "Completed" filter - shows only completed todos
- Click "All" filter - shows all todos
- Demonstrates reactive filtering

### 5. Deleting Todos
- Hover over a todo to reveal the delete button
- Click the × button to delete
- Shows smooth removal animation

### 6. Clear Completed
- Complete multiple todos
- Click "Clear completed" button
- Shows bulk removal of completed items

## Manual Screenshot/GIF Creation

### Using macOS Built-in Tools

1. **Screenshots**:
   - Press `Cmd + Shift + 5` to open screenshot tool
   - Select "Capture Selected Portion"
   - Capture the todo app window

2. **Screen Recording for GIFs**:
   - Press `Cmd + Shift + 5`
   - Select "Record Selected Portion"
   - Record the demo scenario
   - Use tools like Gifski or CloudConvert to convert MOV to GIF

### Using Browser Extensions

1. **Chrome/Firefox**: Install "Awesome Screenshot" or "Nimbus Screenshot"
2. Record specific area of the page
3. Export as GIF

### Using Professional Tools

1. **LICEcap** (Free): Direct GIF recording
2. **Kap** (Free, macOS): Beautiful screen recordings with GIF export
3. **ScreenToGif** (Free, Windows): Record and edit GIFs

## Recommended GIF Settings

- **Size**: 800x600px or smaller for README
- **FPS**: 15-30 for smooth animation
- **Duration**: 5-15 seconds per feature
- **Quality**: Optimize for web (< 5MB per GIF)

## Feature Highlights for README

### Main Features to Showcase

1. **Reactive State Management**
   - Show real-time updates when adding/removing todos
   - Demonstrate filter changes

2. **Beautiful UI**
   - Gradient background
   - Smooth hover effects
   - Modern design with proper spacing

3. **Full CRUD Operations**
   - Create: Adding new todos
   - Read: Displaying todo list
   - Update: Toggling completion
   - Delete: Removing todos

4. **Advanced Features**
   - Filtering system
   - Stats tracking (active/completed counts)
   - Bulk operations (Clear completed)
   - Responsive design

## Example README Section

```markdown
## Examples

### Todo App

A beautiful, fully-featured todo application built with Layer9:

![Layer9 Todo App Demo](examples/todo-app/demo.gif)

**Features:**
- ✅ Add, complete, and delete todos
- 🎨 Beautiful modern UI with gradients
- 🔍 Filter by All/Active/Completed
- 📊 Real-time stats tracking
- 🚀 Lightning-fast WASM performance
- 💾 Persistent storage ready

[Try it live](https://your-demo-url.com) | [View source](examples/todo-app)
```

## Tips for Great Demo GIFs

1. **Clean State**: Start with a fresh browser window
2. **Smooth Actions**: Move the mouse deliberately, not too fast
3. **Pause on Features**: Hold for 1-2 seconds on important UI elements
4. **Show Feedback**: Let animations complete before next action
5. **Loop Friendly**: End state should transition well to start

## File Organization

```
examples/todo-app/
├── demo-gifs/
│   ├── overview.gif      # Complete feature demo
│   ├── add-todo.gif      # Adding todos
│   ├── filters.gif       # Filter functionality
│   └── complete-todo.gif # Completing/deleting
└── screenshots/
    ├── empty-state.png
    ├── with-todos.png
    └── filtered-view.png
```