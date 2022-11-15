# rust_ray

## Project Overview
A little learning test raytracer project in rust.

## Helpful Links
* [shadow-rs](https://github.com/baoyachi/shadow-rs) - used to get build information

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Credits
### Original Code

### Fonts

| Name |     |
|------|-----|
|      |     |
|      |     |
|      |     |
|      |     |
|      |     |


### Crates

| Name                                                                                                     | Purpose                                                                                                                                                                      |
|----------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`shadow-rs`][shadow-rs]                                                                                 | Stores build information for access at runtime                                                                                                                               |
| [`color-eyre`][color-eyre]                                                                               | Enhances panics/errors                                                                                                                                                       |
| [`tracing`][tracing]<br/>[`tracing-subscriber`][tracing-subscriber]<br/>[`tracing-error`][tracing-error] | Trace program execution                                                                                                                                                      |
| [`pretty-assertions`][pretty-assertions]                                                                 | Makes assertion failures colourful and shows a diff                                                                                                                          |
| [`indoc`][indoc]                                                                                         | Automatically de-indents strings (if the IDE tries to line things up)                                                                                                        |
| [`clipboard`][clipboard]                                                                                 | Can you guess?                                                                                                                                                               |
| [`imgui`][imgui]                                                                                         | Immediate-mode Graphical User Interface (ImGUI) - makes pretty stuff appear on screen really easily. Technically just a wrapper for the C++ library [Dear ImGui][dear-imgui] |
| [`glium`][glium]                                                                                         | OpenGL wrapper (used to create an OpenGL context for the ImGUI)                                                                                                              |
| [`imgui-glium-renderer`][imgui-glium-renderer]                                                           | Provides a renderer for [`imgui`][imgui] (does the actual rendering of stuff)                                                                                                |
| [`imgui-winit-support`][imgui-winit-support]                                                             | Provides a backend for [`imgui`][imgui] (handles window management, IO, etc)                                                                                                 |
|                                                                                                          |                                                                                                                                                                              |

[shadow-rs]: https://docs.rs/crate/shadow-rs
[color-eyre]: https://docs.rs/crate/color-eyre
[tracing]: https://docs.rs/crate/tracing
[tracing-subscriber]: https://docs.rs/crate/tracing-subscriber
[tracing-error]: https://docs.rs/crate/tracing-error
[indoc]: https://docs.rs/crate/indoc
[pretty-assertions]: https://docs.rs/crate/pretty-assertions
[clipboard]: https://docs.rs/crate/clipboard
[imgui]: https://docs.rs/crate/imgui
[glium]: https://docs.rs/crate/glium
[imgui-glium-renderer]: https://docs.rs/crate/imgui-glium-renderer
[imgui-winit-support]: https://docs.rs/crate/imgui-winit
[dear-imgui]: https://github.com/ocornut/imgui
