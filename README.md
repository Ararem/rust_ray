# rust_ray

## Project Overview
A little learning test raytracer project in rust.

## Helpful Links
* [Inigo Quilez](https://iquilezles.org/) An *AMAZINGLY* helpful author of many shadertoy shaders, and very helpful code/tutorials. 
* [RayTracing in a weekend](https://in1weekend.blogspot.com/) One of the best resources for getting a raytracer built, goes through many of the steps in detail and gives you a fully functional tracer at the end!

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

### Original Code (aka Authors)
* @Ararem <48875125+ararem@users.noreply.github.com> - Original author/owner

### Fonts

These are the default fonts provided with the application. They are copied to the output directory and loaded at runtime

* Consolas
* Fira Code
* JetBrains Mono
* Roboto
* Source Code Pro
* Metropolis

### Crates

| Name                                                                                                     | Purpose                                                                                                                                                                      |
|----------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`shadow-rs`][shadow-rs]                                                                                 | Stores build information for access at runtime                                                                                                                               |
| [`color-eyre`][color-eyre]                                                                               | Enhances panics/errors                                                                                                                                                       |
| [`pretty-assertions`][pretty-assertions]                                                                 | Makes assertion failures colourful and shows a diff                                                                                                                          |
| [`lazy_static`][lazy_static]                                                                             | Workaround to allow initialising `static` variables with a non-const initialiser, by making them lazily evaluate at deref time                                               |
| [`nameof`][nameof]                                                                                       | Little macro hack to return the name of a variable/type/field. Makes refactoring much easier as it's an actual reference not a string                                        |
| [`itertools`][itertools]                                                                                 | Extends the standard library support for iterators                                                                                                                           |
| [`backtrace`][backtrace]                                                                                 | Allows for capturing backtraces (aka stack traces in other languages)                                                                                                        |
|                                                                                                          |                                                                                                                                                                              |
| [`tracing`][tracing]<br/>[`tracing-subscriber`][tracing-subscriber]<br/>[`tracing-error`][tracing-error] | Trace program execution                                                                                                                                                      |
|                                                                                                          |                                                                                                                                                                              |
| [`tracing-flame`][tracing-flame]                                                                         | Generates flamegraphs from [`tracing`][tracing] spantraces                                                                                                                   |
| [`criterion`][criterion]                                                                                 | Performance profiling toolkit library, with proper statistics                                                                                                                |
|                                                                                                          |                                                                                                                                                                              |
| [`indoc`][indoc]                                                                                         | Automatically de-indents strings (if the IDE tries to line things up). Makes multiline strings much easier to work with in source                                            |
| [`regex`][regex]                                                                                         | Provides support for...... Regex!!                                                                                                                                           |
| [`fancy-regex`][fancy-regex]                                                                             | Provides support for...... Regex!! But much more betterer than [`regex`][regex]                                                                                              |
| [`fs_extra`][fs_extra]                                                                                   | Extends the standard library support for filesystem stuff                                                                                                                    |
| [`path-clean`][path-clean]                                                                               | Cleans up file paths, and removes redundant parts                                                                                                                            |
| [`slice-deque`][slice-deque]                                                                             | Implementation of [alloc::collections::vec_deque::VecDeque] that can be directly mapped to a slice                                                                           |
| [`multiqueue2`][multiqueue2]                                                                             | Fork of `multiqueue`. Allows for inter-thread messaging with multiple senders and receivers all bundled together                                                             |
| [`clipboard`][clipboard]                                                                                 | Can you guess?                                                                                                                                                               |
| [`rand`][rand]                                                                                           | See above                                                                                                                                                                    |
| [`humantime`][humantime]                                                                                 | Makes the `std::time` structs much more readable for us poor humans when formatted                                                                                           |
| [`serde`][serde]                                                                                         | Magically de/serialises rust structs to and from a load of different formats, like `JSON`, `RON`, `YAML`, `TOML`, etc                                                        |
| [`mint`][mint]                                                                                           | Interoperability standard for mathematical numeric types                                                                                                                     |
| [`throttle`][throttle]                                                                                   | Tiny little library that can be used to throttle things                                                                                                                      |
|                                                                                                          |                                                                                                                                                                              |
| [`imgui`][imgui]                                                                                         | Immediate-mode Graphical User Interface (ImGUI) - makes pretty stuff appear on screen really easily. Technically just a wrapper for the C++ library [Dear ImGui][dear-imgui] |
| [`glium`][glium]                                                                                         | OpenGL wrapper (used to create an OpenGL context for the ImGUI)                                                                                                              |
| [`imgui-glium-renderer`][imgui-glium-renderer]                                                           | Provides a renderer for [`imgui`][imgui] (does the actual rendering of stuff)                                                                                                |
| [`imgui-winit-support`][imgui-winit-support]                                                             | Provides a backend for [`imgui`][imgui] (handles window management, IO, etc)                                                                                                 |
| [`winit`][winit]                                                                                         | OS Window interaction library. Required by [`imgui-winit-support`][imgui-winit-support]                                                                                      |
|                                                                                                          |                                                                                                                                                                              |
| [`vek`][vek]                                                                                             | > Generic 2D-3D math swiss army knife for game engines, with SIMD support and focus on convenience.                                                                          |

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
[lazy_static]: https://docs.rs/crate/lazy_static
[regex]: https://docs.rs/crate/regex
[fs_extra]: https://docs.rs/crate/fs_extra
[itertools]: https://docs.rs/crate/itertools
[nameof]: https://docs.rs/crate/nameof
[backtrace]: https://docs.rs/crate/backtrace
[tracing-flame]: https://docs.rs/crate/tracing-flame
[criterion]: https://docs.rs/crate/criterion
[fancy-regex]: https://docs.rs/crate/fancy-regex
[path-clean]: https://docs.rs/crate/path-clean
[slice-deque]: https://docs.rs/crate/slice-deque
[multiqueue2]: https://docs.rs/crate/multiqueue2
[multiqueue]: https://docs.rs/crate/multiqueue
[rand]: https://docs.rs/crate/rand
[humantime]: https://docs.rs/crate/humantime
[serde]: https://docs.rs/crate/serde
[mint]: https://docs.rs/crate/mint
[throttle]: https://docs.rs/crate/throttle
[winit]: https://docs.rs/crate/winit
[vek]: https://docs.rs/crate/vek