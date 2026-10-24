# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/r4ai/shadcn-iced/compare/v0.2.0...v0.3.0) - 2026-10-24

### Added

- [**breaking**] make every component work with touch and the keyboard ([#70](https://github.com/r4ai/shadcn-iced/pull/70))
- [**breaking**] close remaining shadcn/ui differences and fix animation and behavior bugs ([#68](https://github.com/r4ai/shadcn-iced/pull/68))

### Fixed

- keep charts and other clipped layers under headers, and stop the settings preview from shifting the page ([#74](https://github.com/r4ai/shadcn-iced/pull/74))

### Other

- add a README to each published crate ([#72](https://github.com/r4ai/shadcn-iced/pull/72))

## [0.2.0](https://github.com/r4ai/shadcn-iced/compare/v0.1.0...v0.2.0) - 2026-10-18

### Added

- *(shadcn_iced_core)* share a dashed border primitive ([#57](https://github.com/r4ai/shadcn-iced/pull/57))
- *(shadcn_iced_core)* highlight the syntax of code blocks ([#56](https://github.com/r4ai/shadcn-iced/pull/56))
- *(shadcn_iced)* balance the lines of alert dialog and drawer descriptions ([#54](https://github.com/r4ai/shadcn-iced/pull/54))
- *(shadcn_iced)* let buttons narrow their padding for custom icons and override their text color ([#53](https://github.com/r4ai/shadcn-iced/pull/53))
- *(shadcn_iced_core)* let inputs follow `text-base md:text-sm` automatically ([#52](https://github.com/r4ai/shadcn-iced/pull/52))
- *(shadcn_iced)* let cards replace the padding of their content ([#55](https://github.com/r4ai/shadcn-iced/pull/55))
- *(shadcn_iced_core)* move the behavior of the remaining components into primitives ([#46](https://github.com/r4ai/shadcn-iced/pull/46))
- bring the chart up to par with shadcn/ui ([#45](https://github.com/r4ai/shadcn-iced/pull/45))
- *(shadcn_iced)* make buttons focusable with the keyboard ([#44](https://github.com/r4ai/shadcn-iced/pull/44))
- *(shadcn_iced_core)* [**breaking**] scroll smoothly like Chrome, as an opt-in ([#42](https://github.com/r4ai/shadcn-iced/pull/42))

### Fixed

- *(shadcn_iced_core)* draw Tailwind's layered shadows with their spreads ([#50](https://github.com/r4ai/shadcn-iced/pull/50))
- return the focus to the trigger after an overlay closes ([#47](https://github.com/r4ai/shadcn-iced/pull/47))
- *(shadcn_iced_core)* lay out the end of an animated width ([#59](https://github.com/r4ai/shadcn-iced/pull/59))
- *(shadcn_iced_core)* let a submenu open again after one of its items was selected ([#48](https://github.com/r4ai/shadcn-iced/pull/48))

### Other

- *(shadcn_iced)* build only the shown tooltip of a chart ([#67](https://github.com/r4ai/shadcn-iced/pull/67))
- [**breaking**] stop bundling Inter by default, and forward the font features from shadcn-iced ([#66](https://github.com/r4ai/shadcn-iced/pull/66))
- *(shadcn_iced)* move the chart tooltip without laying out the window ([#63](https://github.com/r4ai/shadcn-iced/pull/63))
- *(shadcn_iced)* stop the pulse of skeletons when they are hidden or animations are off ([#61](https://github.com/r4ai/shadcn-iced/pull/61))
- *(shadcn_iced)* lay out only the shown tooltip of a chart ([#60](https://github.com/r4ai/shadcn-iced/pull/60))
- *(shadcn_iced)* redraw sliders and accordion triggers only when their hover changes ([#58](https://github.com/r4ai/shadcn-iced/pull/58))
- *(shadcn_iced_core)* cache the tokens of recent themes and look up faded themes without allocating ([#65](https://github.com/r4ai/shadcn-iced/pull/65))
- *(shadcn_iced_core)* skip the focus query of FocusWithin on pointer moves ([#64](https://github.com/r4ai/shadcn-iced/pull/64))
- *(shadcn_iced_core)* redraw link underlines only when their hover changes ([#62](https://github.com/r4ai/shadcn-iced/pull/62))

## [0.1.0](https://github.com/r4ai/shadcn-iced/compare/v0.0.0...v0.1.0) - 2026-10-18

### Added

- smooth interactions in components and a charts showcase ([#26](https://github.com/r4ai/shadcn-iced/pull/26))
- animate sonner toasts like sonner ([#23](https://github.com/r4ai/shadcn-iced/pull/23))
- compare every component with shadcn/ui and fix the differences ([#5](https://github.com/r4ai/shadcn-iced/pull/5))

### Other

- rebuild CI with least privilege and parallel jobs, and automate releases ([#34](https://github.com/r4ai/shadcn-iced/pull/34))
- move the toast stack and the chart geometry into shadcn_iced_core ([#27](https://github.com/r4ai/shadcn-iced/pull/27))
- rename the project to shadcn-iced
- harden GitHub Actions against supply-chain attacks ([#1](https://github.com/r4ai/shadcn-iced/pull/1))
