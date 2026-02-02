# Performance Monitoring

Anteater includes a built-in performance overlay to help you monitor rendering performance and detect performance regressions during development.

## Performance Overlay

### Enabling the Overlay

Press **F12** to toggle the performance overlay on/off.

The overlay appears in the top-right corner and displays:

```
┌─────────────────────────────┐
│ Performance Metrics         │
├─────────────────────────────┤
│ FPS: 60.0                   │  (Green: ≥55, Yellow: 30-55, Red: <30)
│ Frame Time (avg): 16.67 ms  │
│ Frame Time (min): 14.23 ms  │
│ Frame Time (max): 18.92 ms  │
├─────────────────────────────┤
│ Memory: 2.3 MB              │
├─────────────────────────────┤
│ Frame Time Graph:           │
│ [Real-time graph showing    │
│  frame times over last 2s]  │
│                             │
│ Green line = 60fps target   │
└─────────────────────────────┘
```

### Metrics Explained

**FPS (Frames Per Second)**
- Calculated as rolling average over last 2 seconds (120 frames)
- Color-coded for quick assessment:
  - **Green**: ≥55 fps (excellent)
  - **Yellow**: 30-55 fps (acceptable but not ideal)
  - **Red**: <30 fps (performance problem)

**Frame Time (avg)**
- Average time to render one frame in milliseconds
- Target: ≤16.67ms (60fps) when vsync is enabled
- If consistently higher, indicates performance bottleneck

**Frame Time (min/max)**
- Shows frame time variance
- Large difference between min/max indicates frame time inconsistency
- Spikes (high max) can cause visual stuttering

**Frame Time Graph**
- Real-time visualization of last 120 frames
- Horizontal green line shows 16.67ms target (60fps)
- Frames above the line are slower than 60fps
- Consistent height = stable performance
- Spikes = performance hitches

**Memory**
- Approximate memory usage of egui context
- Does not include all application memory
- Useful for detecting memory leaks during development

## VSync Configuration

### What is VSync?

**VSync (Vertical Synchronization)** synchronizes frame rendering with your display's refresh rate:

- **Enabled** (default): FPS capped at display refresh rate (usually 60Hz/60fps)
  - Prevents screen tearing
  - Consistent frame times
  - Lower power consumption
  - **This is what users should experience**

- **Disabled**: Uncapped FPS
  - Useful for testing maximum rendering performance
  - Can cause screen tearing
  - Higher power consumption
  - **Only use for development/benchmarking**

### Changing VSync Setting

Edit `crates/anteater/src/main.rs`:

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([1600.0, 900.0])
        .with_title("Anteater Debugger"),

    vsync: true,  // <-- Change this to false to disable vsync

    hardware_acceleration: eframe::HardwareAcceleration::Preferred,

    ..Default::default()
};
```

### When to Disable VSync

**Only disable vsync when:**
1. Testing maximum achievable FPS
2. Profiling rendering performance
3. Comparing performance changes (new code vs. old code)
4. Debugging frame pacing issues

**Never ship with vsync disabled** - it wastes power and causes screen tearing.

## Performance Requirements

From `ARCHITECTURE.md`:

> Performance is non-negotiable. A debugger that stutters or lags breaks flow state.
> Anteater must feel instant—UI at 60fps minimum, operations completing in milliseconds.

### Target Metrics

**With VSync Enabled (Production):**
- FPS: 60 (locked to display refresh rate)
- Frame time: ≤16.67ms average
- Frame time variance: <2ms (min to max difference)
- No dropped frames during normal operation

**With VSync Disabled (Testing):**
- FPS: >100 (indicates rendering headroom)
- Frame time: <10ms average
- If FPS <100 with vsync off, investigate performance

### Common Performance Issues

**If FPS drops below 55:**

1. **Check panel content**
   - Memory panel showing too many bytes? Reduce visible rows
   - Variables panel deeply nested? Limit expansion depth
   - Disassembly panel showing too many instructions?

2. **Check for allocations in render loop**
   - String formatting in hot path
   - Creating new collections every frame
   - Unnecessary clones

3. **Check egui debug painter**
   - `ctx.debug_on_hover()` to see what's being drawn
   - Look for excessive widget count

4. **Use the graph to identify patterns**
   - Periodic spikes? Likely GC or background task
   - Gradual slowdown? Likely memory leak
   - Consistent slow? Rendering bottleneck

## Development Workflow

### Testing Performance Impact of Changes

```bash
# Before making changes:
1. Enable performance overlay (F12)
2. Note average FPS and frame time
3. Run typical workflow (step through code, inspect variables, etc.)
4. Note min/max frame times

# After making changes:
5. Repeat same workflow
6. Compare metrics
7. If FPS dropped >5 or frame time increased >2ms, investigate

# For detailed profiling:
8. Disable vsync (see above)
9. Compare FPS with vsync disabled
10. Re-enable vsync before committing
```

### Acceptable Performance Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| Idle (no interaction) | 60 fps | - |
| Scrolling memory panel | 60 fps | 1-2 dropped frames acceptable |
| Expanding large struct | 60 fps | Single frame drop acceptable |
| Switching themes | - | <100ms to apply |
| Opening new panel | - | <50ms |
| Resizing window | 60 fps | - |
| Dragging to rearrange panels | 60 fps | - |

## Keyboard Shortcuts Summary

- **F12**: Toggle performance overlay
- Use whenever:
  - Developing new UI features
  - Testing performance changes
  - Debugging frame drops
  - Validating performance requirements

## Future Enhancements

Potential improvements to performance monitoring:

1. **Persistent logging**
   - Save performance metrics to file
   - Analyze performance over time
   - Detect regressions in CI

2. **Profiler integration**
   - Click "Profile" button to capture 5 seconds
   - Show breakdown by panel
   - Identify slow widgets

3. **Performance budget**
   - Set per-panel frame time budgets
   - Warning when budget exceeded
   - Automated performance testing

4. **GPU metrics**
   - GPU utilization
   - VRAM usage
   - Draw call count

5. **Async operation tracking**
   - Time spent in debug engine
   - Memory reads/writes count
   - ptrace syscall latency

---

**Remember**: The performance overlay is a development tool. It will not be shown to users by default. Use it to ensure Anteater stays fast and responsive.
