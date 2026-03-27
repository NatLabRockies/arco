# Changelog

## [0.2.5](https://github.com/NatLabRockies/arco/compare/arco-v0.2.5...arco-v0.2.5) (2026-03-27)


### Features

* adding ipopt support for non-linear stuff ([#40](https://github.com/NatLabRockies/arco/issues/40)) ([2cd81ba](https://github.com/NatLabRockies/arco/commit/2cd81badfeffa55979e33106d1b3426287e62cdc))
* **api:** ergonomic positional args for add_variables ([#45](https://github.com/NatLabRockies/arco/issues/45)) ([efb3cac](https://github.com/NatLabRockies/arco/commit/efb3cacd8836b2b9aa9d9a122f5c74b2f4ce53f1))
* **Xpress:** Adding xpress support. ([#44](https://github.com/NatLabRockies/arco/issues/44)) ([a26c05d](https://github.com/NatLabRockies/arco/commit/a26c05d68d4688fac810277453e7160efb6e562d))


### Bug Fixes

* adding CI gates  ([#68](https://github.com/NatLabRockies/arco/issues/68)) ([81c298f](https://github.com/NatLabRockies/arco/commit/81c298f87da7cea30b5972597cc2925a528be2f4))
* Adding macos wheels ([#17](https://github.com/NatLabRockies/arco/issues/17)) ([101d19c](https://github.com/NatLabRockies/arco/commit/101d19c4ebd2a38fa0f3b104515271ce98ffa59b))
* Adding missing operators for Variables. ([#19](https://github.com/NatLabRockies/arco/issues/19)) ([f288e2d](https://github.com/NatLabRockies/arco/commit/f288e2d0e62725190b287710d9af94c30adfb24e))
* adding new workflows ([33a1d62](https://github.com/NatLabRockies/arco/commit/33a1d624b7b41a1123bb0e2be7ddc3d816333f75))
* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* **arco-highs:** harden FFI solution extraction error handling ([#29](https://github.com/NatLabRockies/arco/issues/29)) ([9f759a3](https://github.com/NatLabRockies/arco/commit/9f759a3c6daf78ae5b5bb105cf705eb49958eb7c))
* **ci:** dispatch release workflows and fix CLI build quoting ([#59](https://github.com/NatLabRockies/arco/issues/59)) ([9e8838c](https://github.com/NatLabRockies/arco/commit/9e8838cb69fddb530f46c7149f57d13a70b75b49))
* **ci:** isolate cargo-dist CLI release builds from workspace crates ([#61](https://github.com/NatLabRockies/arco/issues/61)) ([5c6ae65](https://github.com/NatLabRockies/arco/commit/5c6ae659c7f1426a79dfa7c74d3ecb3c4e15a065))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **ci:** stabilize unified python+cli release workflow ([#56](https://github.com/NatLabRockies/arco/issues/56)) ([f22f07c](https://github.com/NatLabRockies/arco/commit/f22f07c8ea441436640fabc9c93d1271364b4565))
* **ci:** trigger release smoke for release-please PRs ([#63](https://github.com/NatLabRockies/arco/issues/63)) ([28b876a](https://github.com/NatLabRockies/arco/commit/28b876ab74479744df7084a55ec065405e31e2b5))
* **kdl:** replace let-chains for Rust 1.85 compatibility ([#64](https://github.com/NatLabRockies/arco/issues/64)) ([ceb558a](https://github.com/NatLabRockies/arco/commit/ceb558a7624818dae2380c7ac1a6e7eb48306c51))
* key error ([#37](https://github.com/NatLabRockies/arco/issues/37)) ([8304fc3](https://github.com/NatLabRockies/arco/commit/8304fc3bb6e1e0539fb926e59811081a2a196113))
* linter errors not founding the module for type hints ([#15](https://github.com/NatLabRockies/arco/issues/15)) ([019ad25](https://github.com/NatLabRockies/arco/commit/019ad25a55e2358ee69baf7d910b9508be3bbd3d))
* objective function heavy operation ([#27](https://github.com/NatLabRockies/arco/issues/27)) ([9d337e1](https://github.com/NatLabRockies/arco/commit/9d337e1711c31d5211adead79cfff34823819b73))
* **release:** correct grouped PR title pattern ([#70](https://github.com/NatLabRockies/arco/issues/70)) ([b754244](https://github.com/NatLabRockies/arco/commit/b7542448bc2e59bede5587657a4afe3444aaf7e7))
* **release:** disable draft releases to stop stale release loops ([28c8f7a](https://github.com/NatLabRockies/arco/commit/28c8f7a3eaeab85226373edbc6a3fd51a6333d91))
* **release:** match tag format to existing tags ([#72](https://github.com/NatLabRockies/arco/issues/72)) ([3a0553c](https://github.com/NatLabRockies/arco/commit/3a0553cbe98ea42a13f87bf1864440297d5d755c))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))
* removing duplication code path for solver kwargs ([#23](https://github.com/NatLabRockies/arco/issues/23)) ([c96c3f0](https://github.com/NatLabRockies/arco/commit/c96c3f0506ca017643463ef5511cd11adbb5ca50))
* unified cli releases ([#57](https://github.com/NatLabRockies/arco/issues/57)) ([6cbf8a1](https://github.com/NatLabRockies/arco/commit/6cbf8a1783fb292743f9e45e7db0bab62c2816ad))
* **xpress:** update FFI bindings and licensing for Xpress SDK 9+ ([#52](https://github.com/NatLabRockies/arco/issues/52)) ([a4859b3](https://github.com/NatLabRockies/arco/commit/a4859b3017908f6bea8ef2d8b63da7be6a382b63))


### Performance

* improving arco memory management and model built time ([#42](https://github.com/NatLabRockies/arco/issues/42)) ([f034fac](https://github.com/NatLabRockies/arco/commit/f034fac4abf684f53194d800519a769478cb3c4a))
* reducing number of copies from highs ([#28](https://github.com/NatLabRockies/arco/issues/28)) ([7ba4dde](https://github.com/NatLabRockies/arco/commit/7ba4dde83d67bf99fbf213ce99e36fa5ed194b07))


### Refactoring

* **arco-bench:** split benchmark runner into modules ([#36](https://github.com/NatLabRockies/arco/issues/36)) ([6e1b891](https://github.com/NatLabRockies/arco/commit/6e1b891b1faf55b6bebeef823945c256c17e0128))
* **arco-blocks:** centralize runtime error logging helper ([#34](https://github.com/NatLabRockies/arco/issues/34)) ([f87e374](https://github.com/NatLabRockies/arco/commit/f87e374118c11649db2aaa3495bc77b1471cc00d))
* **arco-python:** split array wrappers into modules ([#32](https://github.com/NatLabRockies/arco/issues/32)) ([06554a6](https://github.com/NatLabRockies/arco/commit/06554a6aa7106952edfcff26c343ab7fda0e1345))
* Reducing memory consumption of CRS matrix and adding more options for creating matrix representations ([#24](https://github.com/NatLabRockies/arco/issues/24)) ([bed359e](https://github.com/NatLabRockies/arco/commit/bed359eb9e8319e646b3e658a4caa6e91b176382))


### Documentation

* adding full eexplaination of sdom and solver configuration. ([#53](https://github.com/NatLabRockies/arco/issues/53)) ([b382d39](https://github.com/NatLabRockies/arco/commit/b382d39f07e51311691c4ad62e44e4f2dc93083c))
* Cleaning stuff around. ([#21](https://github.com/NatLabRockies/arco/issues/21)) ([830430e](https://github.com/NatLabRockies/arco/commit/830430e243152e7c7567494e599d4b3f0c2ec764))
* expand public API rustdoc across crates ([#31](https://github.com/NatLabRockies/arco/issues/31)) ([4bfd6f6](https://github.com/NatLabRockies/arco/commit/4bfd6f6929949d372ea70e0582a43bfe68468c56))


### Build

* better ci management ([#38](https://github.com/NatLabRockies/arco/issues/38)) ([0489258](https://github.com/NatLabRockies/arco/commit/0489258f876b938fd74542482af2513eccd8735f))
* **ci:** improving ci ([#67](https://github.com/NatLabRockies/arco/issues/67)) ([a6bc40b](https://github.com/NatLabRockies/arco/commit/a6bc40bcfb95f2abe35632677d588afb5202829a))
* **deps:** bump actions/download-artifact from 7.0.0 to 8.0.0 ([#47](https://github.com/NatLabRockies/arco/issues/47)) ([6983021](https://github.com/NatLabRockies/arco/commit/6983021af8ba8b0b1a04f34ce3590a31c166a863))
* **deps:** bump actions/upload-artifact from 6.0.0 to 7.0.0 ([#49](https://github.com/NatLabRockies/arco/issues/49)) ([28c0dd5](https://github.com/NatLabRockies/arco/commit/28c0dd5e30dd0473bdb735989e88c52a505d9538))
* **deps:** bump PyO3/maturin-action from 1.50.0 to 1.50.1 ([#48](https://github.com/NatLabRockies/arco/issues/48)) ([4e498c8](https://github.com/NatLabRockies/arco/commit/4e498c8a7dd935352f621e43b106790c1fcd8d8a))


### Chores

* **release:** force 0.1.1 ([6053cbb](https://github.com/NatLabRockies/arco/commit/6053cbb52d837106e2b82eb7cba50c5de76efdb9))


### Tests

* **arco-core:** add direct coverage for types and slack helpers ([#33](https://github.com/NatLabRockies/arco/issues/33)) ([164dcfb](https://github.com/NatLabRockies/arco/commit/164dcfb70fdd94f2cbb98cf263c06790d56b1d05))

## [0.2.5](https://github.com/NatLabRockies/arco/compare/arco-v0.2.4...arco-v0.2.5) (2026-03-27)


### Bug Fixes

* adding CI gates  ([#68](https://github.com/NatLabRockies/arco/issues/68)) ([81c298f](https://github.com/NatLabRockies/arco/commit/81c298f87da7cea30b5972597cc2925a528be2f4))
* **release:** correct grouped PR title pattern ([#70](https://github.com/NatLabRockies/arco/issues/70)) ([b754244](https://github.com/NatLabRockies/arco/commit/b7542448bc2e59bede5587657a4afe3444aaf7e7))
* **release:** match tag format to existing tags ([#72](https://github.com/NatLabRockies/arco/issues/72)) ([3a0553c](https://github.com/NatLabRockies/arco/commit/3a0553cbe98ea42a13f87bf1864440297d5d755c))


### Build

* **ci:** improving ci ([#67](https://github.com/NatLabRockies/arco/issues/67)) ([a6bc40b](https://github.com/NatLabRockies/arco/commit/a6bc40bcfb95f2abe35632677d588afb5202829a))

## [0.2.4](https://github.com/NatLabRockies/arco/compare/arco-v0.2.3...arco-v0.2.4) (2026-03-24)


### Bug Fixes

* **ci:** isolate cargo-dist CLI release builds from workspace crates ([#61](https://github.com/NatLabRockies/arco/issues/61)) ([5c6ae65](https://github.com/NatLabRockies/arco/commit/5c6ae659c7f1426a79dfa7c74d3ecb3c4e15a065))
* **ci:** trigger release smoke for release-please PRs ([#63](https://github.com/NatLabRockies/arco/issues/63)) ([28b876a](https://github.com/NatLabRockies/arco/commit/28b876ab74479744df7084a55ec065405e31e2b5))
* **kdl:** replace let-chains for Rust 1.85 compatibility ([#64](https://github.com/NatLabRockies/arco/issues/64)) ([ceb558a](https://github.com/NatLabRockies/arco/commit/ceb558a7624818dae2380c7ac1a6e7eb48306c51))

## [0.2.3](https://github.com/NatLabRockies/arco/compare/arco-v0.2.2...arco-v0.2.3) (2026-03-23)


### Bug Fixes

* **ci:** dispatch release workflows and fix CLI build quoting ([#59](https://github.com/NatLabRockies/arco/issues/59)) ([9e8838c](https://github.com/NatLabRockies/arco/commit/9e8838cb69fddb530f46c7149f57d13a70b75b49))

## [0.2.2](https://github.com/NatLabRockies/arco/compare/arco-v0.2.1...arco-v0.2.2) (2026-03-23)


### Bug Fixes

* unified cli releases ([#57](https://github.com/NatLabRockies/arco/issues/57)) ([6cbf8a1](https://github.com/NatLabRockies/arco/commit/6cbf8a1783fb292743f9e45e7db0bab62c2816ad))

## [0.2.1](https://github.com/NatLabRockies/arco/compare/arco-v0.2.0...arco-v0.2.1) (2026-03-22)


### Bug Fixes

* **ci:** stabilize unified python+cli release workflow ([#56](https://github.com/NatLabRockies/arco/issues/56)) ([f22f07c](https://github.com/NatLabRockies/arco/commit/f22f07c8ea441436640fabc9c93d1271364b4565))
* **xpress:** update FFI bindings and licensing for Xpress SDK 9+ ([#52](https://github.com/NatLabRockies/arco/issues/52)) ([a4859b3](https://github.com/NatLabRockies/arco/commit/a4859b3017908f6bea8ef2d8b63da7be6a382b63))


### Documentation

* adding full eexplaination of sdom and solver configuration. ([#53](https://github.com/NatLabRockies/arco/issues/53)) ([b382d39](https://github.com/NatLabRockies/arco/commit/b382d39f07e51311691c4ad62e44e4f2dc93083c))


### Build

* **deps:** bump actions/download-artifact from 7.0.0 to 8.0.0 ([#47](https://github.com/NatLabRockies/arco/issues/47)) ([6983021](https://github.com/NatLabRockies/arco/commit/6983021af8ba8b0b1a04f34ce3590a31c166a863))
* **deps:** bump actions/upload-artifact from 6.0.0 to 7.0.0 ([#49](https://github.com/NatLabRockies/arco/issues/49)) ([28c0dd5](https://github.com/NatLabRockies/arco/commit/28c0dd5e30dd0473bdb735989e88c52a505d9538))
* **deps:** bump PyO3/maturin-action from 1.50.0 to 1.50.1 ([#48](https://github.com/NatLabRockies/arco/issues/48)) ([4e498c8](https://github.com/NatLabRockies/arco/commit/4e498c8a7dd935352f621e43b106790c1fcd8d8a))

## [0.2.0](https://github.com/NatLabRockies/arco/compare/arco-v0.1.7...arco-v0.2.0) (2026-02-25)


### Features

* adding ipopt support for non-linear stuff ([#40](https://github.com/NatLabRockies/arco/issues/40)) ([2cd81ba](https://github.com/NatLabRockies/arco/commit/2cd81badfeffa55979e33106d1b3426287e62cdc))
* **api:** ergonomic positional args for add_variables ([#45](https://github.com/NatLabRockies/arco/issues/45)) ([efb3cac](https://github.com/NatLabRockies/arco/commit/efb3cacd8836b2b9aa9d9a122f5c74b2f4ce53f1))
* **Xpress:** Adding xpress support. ([#44](https://github.com/NatLabRockies/arco/issues/44)) ([a26c05d](https://github.com/NatLabRockies/arco/commit/a26c05d68d4688fac810277453e7160efb6e562d))


### Performance

* improving arco memory management and model built time ([#42](https://github.com/NatLabRockies/arco/issues/42)) ([f034fac](https://github.com/NatLabRockies/arco/commit/f034fac4abf684f53194d800519a769478cb3c4a))

## [0.1.7](https://github.com/NatLabRockies/arco/compare/arco-v0.1.6...arco-v0.1.7) (2026-02-21)


### Bug Fixes

* **arco-highs:** harden FFI solution extraction error handling ([#29](https://github.com/NatLabRockies/arco/issues/29)) ([9f759a3](https://github.com/NatLabRockies/arco/commit/9f759a3c6daf78ae5b5bb105cf705eb49958eb7c))
* key error ([#37](https://github.com/NatLabRockies/arco/issues/37)) ([8304fc3](https://github.com/NatLabRockies/arco/commit/8304fc3bb6e1e0539fb926e59811081a2a196113))
* objective function heavy operation ([#27](https://github.com/NatLabRockies/arco/issues/27)) ([9d337e1](https://github.com/NatLabRockies/arco/commit/9d337e1711c31d5211adead79cfff34823819b73))
* removing duplication code path for solver kwargs ([#23](https://github.com/NatLabRockies/arco/issues/23)) ([c96c3f0](https://github.com/NatLabRockies/arco/commit/c96c3f0506ca017643463ef5511cd11adbb5ca50))


### Performance

* reducing number of copies from highs ([#28](https://github.com/NatLabRockies/arco/issues/28)) ([7ba4dde](https://github.com/NatLabRockies/arco/commit/7ba4dde83d67bf99fbf213ce99e36fa5ed194b07))


### Refactoring

* **arco-bench:** split benchmark runner into modules ([#36](https://github.com/NatLabRockies/arco/issues/36)) ([6e1b891](https://github.com/NatLabRockies/arco/commit/6e1b891b1faf55b6bebeef823945c256c17e0128))
* **arco-blocks:** centralize runtime error logging helper ([#34](https://github.com/NatLabRockies/arco/issues/34)) ([f87e374](https://github.com/NatLabRockies/arco/commit/f87e374118c11649db2aaa3495bc77b1471cc00d))
* **arco-python:** split array wrappers into modules ([#32](https://github.com/NatLabRockies/arco/issues/32)) ([06554a6](https://github.com/NatLabRockies/arco/commit/06554a6aa7106952edfcff26c343ab7fda0e1345))
* Reducing memory consumption of CRS matrix and adding more options for creating matrix representations ([#24](https://github.com/NatLabRockies/arco/issues/24)) ([bed359e](https://github.com/NatLabRockies/arco/commit/bed359eb9e8319e646b3e658a4caa6e91b176382))


### Documentation

* Cleaning stuff around. ([#21](https://github.com/NatLabRockies/arco/issues/21)) ([830430e](https://github.com/NatLabRockies/arco/commit/830430e243152e7c7567494e599d4b3f0c2ec764))
* expand public API rustdoc across crates ([#31](https://github.com/NatLabRockies/arco/issues/31)) ([4bfd6f6](https://github.com/NatLabRockies/arco/commit/4bfd6f6929949d372ea70e0582a43bfe68468c56))


### Build

* better ci management ([#38](https://github.com/NatLabRockies/arco/issues/38)) ([0489258](https://github.com/NatLabRockies/arco/commit/0489258f876b938fd74542482af2513eccd8735f))


### Tests

* **arco-core:** add direct coverage for types and slack helpers ([#33](https://github.com/NatLabRockies/arco/issues/33)) ([164dcfb](https://github.com/NatLabRockies/arco/commit/164dcfb70fdd94f2cbb98cf263c06790d56b1d05))

## [0.1.6](https://github.com/NatLabRockies/arco/compare/arco-v0.1.5...arco-v0.1.6) (2026-02-16)


### Bug Fixes

* Adding missing operators for Variables. ([#19](https://github.com/NatLabRockies/arco/issues/19)) ([f288e2d](https://github.com/NatLabRockies/arco/commit/f288e2d0e62725190b287710d9af94c30adfb24e))

## [0.1.5](https://github.com/NatLabRockies/arco/compare/arco-v0.1.4...arco-v0.1.5) (2026-02-13)


### Bug Fixes

* Adding macos wheels ([#17](https://github.com/NatLabRockies/arco/issues/17)) ([101d19c](https://github.com/NatLabRockies/arco/commit/101d19c4ebd2a38fa0f3b104515271ce98ffa59b))

## [0.1.4](https://github.com/NatLabRockies/arco/compare/arco-v0.1.3...arco-v0.1.4) (2026-02-13)


### Bug Fixes

* linter errors not founding the module for type hints ([#15](https://github.com/NatLabRockies/arco/issues/15)) ([019ad25](https://github.com/NatLabRockies/arco/commit/019ad25a55e2358ee69baf7d910b9508be3bbd3d))

## [0.1.3](https://github.com/NatLabRockies/arco/compare/arco-v0.1.2...arco-v0.1.3) (2026-02-12)


### Bug Fixes

* **release:** disable draft releases to stop stale release loops ([28c8f7a](https://github.com/NatLabRockies/arco/commit/28c8f7a3eaeab85226373edbc6a3fd51a6333d91))

## [0.1.2](https://github.com/NatLabRockies/arco/compare/arco-v0.1.1...arco-v0.1.2) (2026-02-12)


### Bug Fixes

* adding new workflows ([33a1d62](https://github.com/NatLabRockies/arco/commit/33a1d624b7b41a1123bb0e2be7ddc3d816333f75))
* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))

## [0.1.1](https://github.com/NatLabRockies/arco/compare/arco-v0.1.2...arco-v0.1.1) (2026-02-12)


### Bug Fixes

* adding new workflows ([33a1d62](https://github.com/NatLabRockies/arco/commit/33a1d624b7b41a1123bb0e2be7ddc3d816333f75))
* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))


### Miscellaneous Chores

* **release:** force 0.1.1 ([6053cbb](https://github.com/NatLabRockies/arco/commit/6053cbb52d837106e2b82eb7cba50c5de76efdb9))

## [0.1.2](https://github.com/NatLabRockies/arco/compare/arco-v0.1.1...arco-v0.1.2) (2026-02-12)


### Bug Fixes

* adding new workflows ([33a1d62](https://github.com/NatLabRockies/arco/commit/33a1d624b7b41a1123bb0e2be7ddc3d816333f75))
* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))

## [0.1.1](https://github.com/NatLabRockies/arco/compare/arco-v0.1.2...arco-v0.1.1) (2026-02-11)


### Bug Fixes

* adding new workflows ([33a1d62](https://github.com/NatLabRockies/arco/commit/33a1d624b7b41a1123bb0e2be7ddc3d816333f75))
* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))


### Miscellaneous Chores

* **release:** force 0.1.1 ([6053cbb](https://github.com/NatLabRockies/arco/commit/6053cbb52d837106e2b82eb7cba50c5de76efdb9))

## [0.1.2](https://github.com/NatLabRockies/arco/compare/arco-v0.1.1...arco-v0.1.2) (2026-02-11)


### Bug Fixes

* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))

## [0.1.1](https://github.com/NatLabRockies/arco/compare/arco-v0.1.0...arco-v0.1.1) (2026-02-11)


### Miscellaneous Chores

* **release:** force 0.1.1 ([6053cbb](https://github.com/NatLabRockies/arco/commit/6053cbb52d837106e2b82eb7cba50c5de76efdb9))
