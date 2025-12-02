# WGPU Migration Notes

## Completed Fixes

### 1. Macroquad Initialization Issues
**Problem:** The game was trying to use macroquad functions (texture loading, material creation) before macroquad was initialized, causing `assertion failed: THREAD_ID.is_some()` panics.

**Solution:** 
- Added `use_wgpu` flag checks to skip macroquad texture loading during initialization
- Modified `Lightmap` to store raw pixel data and optionally create macroquad textures
- Skipped loading of:
  - `weapon_hit_texture_cache`
  - `muzzle_flash_cache`
  - Item models, projectile models, weapon models
  - Tile shaders and border textures
  - Player model textures

### 2. Uniform Buffer Size Mismatch
**Problem:** WGSL shader expected 144 bytes but Rust struct was 128 bytes.

**Solution:** 
- Fixed `LightingUniforms` struct padding to account for WGSL alignment rules
- Vec3 in uniform buffers is aligned to 16 bytes (treated as Vec4)
- Struct must be padded to multiple of 16 bytes (largest alignment)

### 3. Missing COPY_DST Usage Flag
**Problem:** Render target textures were missing `COPY_DST` usage flag, causing validation errors when trying to write texture data.

**Solution:**
- Added `TextureUsages::COPY_DST` to render target creation in `WgpuTexture::create_render_target()`

### 4. macOS Cursor Initialization Crash (SIGBUS)
**Problem:** Intermittent crash during winit event loop initialization on macOS with:
```
Exception Type:    EXC_BAD_ACCESS (SIGBUS)
Exception Subtype: EXC_ARM_DA_ALIGN at 0x000000000bad4007
```

**Root Cause:** Race condition in winit 0.30.12 cursor initialization on macOS. The crash occurs in `ImageIO` when loading cursor images, with a misaligned memory address.

**Workaround (macOS only):**
- Create window as **invisible** with `with_visible(false)`
- Wait 200ms for full system initialization
- Make window visible and set fullscreen after delay
- This completely avoids cursor tracking setup during window creation
- The invisible window prevents winit from calling `reset_cursor_rects` during initialization

**Stack Trace:**
```
winit::platform_impl::macos::view::WinitView::reset_cursor_rects
  -> ImageIO::IIO_Reader::callGetImageCount
    -> crash at 0xbad4007 (misaligned address)
```

**Note:** This is a winit bug, not a bug in our code. 

**Alternative Solutions to Consider:**
1. Update to winit 0.31+ when released (may have fix)
2. Use `ApplicationHandler` trait instead of closure-based event loop (winit 0.30 recommended pattern)
3. Report issue to winit maintainers with crash logs
4. Consider switching to SDL2 or other windowing library if issue persists

**Why This Happens:**
The crash occurs in `ImageIO::IIO_Reader::callGetImageCount` when winit tries to load cursor images during `reset_cursor_rects`. The misaligned address (0xbad4007) suggests memory corruption or use-after-free in the cursor image loading code path. This is likely a timing issue where winit accesses cursor resources before they're fully initialized by the system.

### 5. Camera Using Macroquad Screen Functions
**Problem:** `Camera::follow()`, `Camera::update()` and related methods were calling `screen_width()` and `screen_height()` from macroquad.

**Solution:**
- Added `_with_size` variants of camera methods that accept window dimensions as parameters:
  - `follow_with_size()`
  - `follow_projectile_with_zoom_size()`
  - `update_with_size()`
- Pass window size from `input_state.window_size` to camera methods in `handle_camera()`
- Keep original methods as wrappers that call macroquad for backward compatibility

### 6. Camera Shake Using Macroquad Random
**Problem:** `Camera::update()` was calling `macroquad::rand::gen_range()` for screen shake effects.

**Solution:**
- Replaced `macroquad::rand::gen_range()` with `fastrand::f32()` (already in dependencies)
- Formula: `fastrand::f32() * range * 2.0 - range` to get values in [-range, range]

### 7. HUD Rendering Using Macroquad Draw Functions
**Problem:** All HUD rendering (`HudScoreboard`, debug info, profiler) uses macroquad drawing functions.

**Solution:**
- Wrapped all HUD rendering in `if !self.game_state.use_wgpu` checks
- Skipped rendering of:
  - Player HUD (health, armor, ammo, weapons)
  - Crosshair
  - Scoreboard
  - Debug info and FPS counter
  - Profiler display
- **Note:** HUD will need to be reimplemented with wgpu UI rendering in the future

## Current Status

✅ Game initializes without macroquad
✅ Wgpu renderer creates and renders successfully  
✅ Uniform buffers properly sized
✅ Texture operations work correctly
✅ macOS cursor crash workaround in place (invisible window + 200ms delay)
✅ Camera system works with wgpu (no macroquad screen calls)
✅ Camera shake uses fastrand instead of macroquad
✅ **Game runs successfully with wgpu on macOS!**
✅ Game loop executes without crashes
✅ Deferred lighting system operational

## Known Issues

1. **Cursor crash workaround is a hack** - The invisible window + 200ms delay is not ideal. Monitor winit releases for a proper fix.
2. **Some textures not loaded in wgpu mode** - Models, weapons, etc. will need wgpu texture loading implemented (currently skipped)
3. **Shader rendering disabled in wgpu mode** - Tile shaders and effects need wgpu implementation
4. **HUD/UI not rendered** - All the draw functions still use macroquad and won't render in wgpu mode

## Next Steps

1. Implement wgpu texture loading for models and weapons
2. Port shader rendering system to wgpu
3. Test on other platforms (Windows, Linux)
4. Remove macroquad dependency once wgpu migration is complete
