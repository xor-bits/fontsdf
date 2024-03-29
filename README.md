<div align="center">

# fontsdf

[![dependency status](https://deps.rs/repo/github/Overpeek/fontsdf/status.svg)](https://deps.rs/repo/github/Overpeek/fontsdf)
[![build status](https://github.com/Overpeek/fontsdf/actions/workflows/rust.yml/badge.svg)](https://github.com/Overpeek/fontsdf/actions)
[![crates.io](https://img.shields.io/crates/v/fontsdf.svg?label=fontsdf)](https://crates.io/crates/fontsdf)
[![docs.rs](https://docs.rs/fontsdf/badge.svg)](https://docs.rs/fontsdf/)

</div>

Signed Distance Field (SDF) textures can be used to render text
or other vector art more flexibly[1], with higher quality while
using less video memory (for the texture).

This `no_std` library renders SDF:s **directly** and not by downscaling a higher resolution rasters.

[1] For example:

- It is possible to use single a 64px glyph to render both 14px
  and 200px glyphs.
- It is easy to add glow, outlines and such.

### Comparisons:

#### NOTE: Use fontdue for simple glyphs instead. It is a lot faster.

| Task                         | SDF     | regular  |
|-----------------------------:|:-------:|:--------:|
| High resolution glyphs       | &check; | &check;  |
| Medium resolution glyphs     | &check; | &check;  |
| Low resolution glyphs        |         | &check;  |
| Stretched or squished glyphs | &check; |          |
| Shadows borders and effects  | &check; |          |
| GUI:s                        |         | &check;  |
| 3D game worlds               | &check; |          |

* :white_check_mark: means it is good at

### Example usage with `image`:

```rust
let font = fontsdf::Font::from_bytes(..).unwrap();

let (metrics, sdf) = font.rasterize('x', 64.0, true);
image::GrayImage::from_raw(metrics.width as _, metrics.height as _, sdf)
	.unwrap()
	.save("sdf_x.png")
	.unwrap();
```

### Example output:

#### Normal

<div style="display: flex; align-items: center;">
	<img src="/.github/hash_norm.png"/>
	<img src="/.github/a_norm.png"/>
</div>

#### SDF

<div style="display: flex; align-items: center;">
	<img src="/.github/hash_sdf.png"/>
	<img src="/.github/a_sdf.png"/>
</div>

### Example results:

#### Normal

- 128x156
- 80px font size

<div style="display: flex; align-items: center;">
	<img src="/.github/norm_glyphs.png"/>
	<img src="/.github/norm_text.png"/>
</div>

#### SDF

- 128x128
- 48px (+radius) font size (32px input size should be enough for any output size)
- 'free' shadow

<div style="display: flex; align-items: center;">
	<img src="/.github/sdf_glyphs.png"/>
	<img src="/.github/sdf_text.png"/>
</div>

### TODO:

- dual distance field (https://steamcdn-a.akamaihd.net/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf)
