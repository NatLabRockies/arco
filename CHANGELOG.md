# Changelog

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
