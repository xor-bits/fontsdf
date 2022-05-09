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

[1] For example:

- It is possible to use single a 64px glyph to render both 14px
  and 200px glyphs.
- It is easy to add glow, outlines and such.

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

### TODO:

- dual distance field (https://steamcdn-a.akamaihd.net/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf)
