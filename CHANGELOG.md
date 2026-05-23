# Changelog

## [0.2.5](https://github.com/NatLabRockies/arco/compare/v0.6.1...v0.2.5) (2026-05-23)


### Features

* add projection grammar ([#206](https://github.com/NatLabRockies/arco/issues/206)) ([0e30225](https://github.com/NatLabRockies/arco/commit/0e30225ec3caefffe9dfc33ec2640ef3cdf2cc65))
* adding ipopt support for non-linear stuff ([#40](https://github.com/NatLabRockies/arco/issues/40)) ([2cd81ba](https://github.com/NatLabRockies/arco/commit/2cd81badfeffa55979e33106d1b3426287e62cdc))
* adding pretty printing of sets/variables ([#102](https://github.com/NatLabRockies/arco/issues/102)) ([d56cc33](https://github.com/NatLabRockies/arco/commit/d56cc335b73c384f37f42922ec55f476c2be02f4))
* **api:** ergonomic positional args for add_variables ([#45](https://github.com/NatLabRockies/arco/issues/45)) ([efb3cac](https://github.com/NatLabRockies/arco/commit/efb3cacd8836b2b9aa9d9a122f5c74b2f4ce53f1))
* **bench:** resource-monitor integration for benchmark orchestration ([#159](https://github.com/NatLabRockies/arco/issues/159)) ([#160](https://github.com/NatLabRockies/arco/issues/160)) ([69d05b0](https://github.com/NatLabRockies/arco/commit/69d05b0d56649150ebd96d91c254576a31dadbc3))
* **cli:** add arco --version output ([#214](https://github.com/NatLabRockies/arco/issues/214)) ([124a403](https://github.com/NatLabRockies/arco/commit/124a4034d4cc8a50dbf39f2b56e3d365408c66c9))
* **cli:** add self-update command ([#251](https://github.com/NatLabRockies/arco/issues/251)) ([8de1e2a](https://github.com/NatLabRockies/arco/commit/8de1e2af8e27c3c22ea06bea0718e9b4f584c948))
* **examples:** add minimal ReEDS KDL benchmark ([#246](https://github.com/NatLabRockies/arco/issues/246)) ([9fa1dde](https://github.com/NatLabRockies/arco/commit/9fa1dde240a4779e89cb2d43334706ed28a2a462))
* **examples:** add ReEDS benchmark ([#244](https://github.com/NatLabRockies/arco/issues/244)) ([f1871f5](https://github.com/NatLabRockies/arco/commit/f1871f5a46636190f0137bae72f9369dfef04404))
* expose constraint duals via unified report keyword ([#103](https://github.com/NatLabRockies/arco/issues/103)) ([bdfa40b](https://github.com/NatLabRockies/arco/commit/bdfa40b414e2f23e81498093217100d0674fe785))
* harden API UX ladder across Rust, Python, and docs ([#281](https://github.com/NatLabRockies/arco/issues/281)) ([62bea77](https://github.com/NatLabRockies/arco/commit/62bea77ba8863ff3c4077b939492fe2415b97208))
* implement solver registry architecture core ([#225](https://github.com/NatLabRockies/arco/issues/225)) ([fa16aa4](https://github.com/NatLabRockies/arco/commit/fa16aa47afbc0d69b74b271a71eac39c4c88c760))
* **kdl:** add scoped inferred IDs for tuple diagnostics ([#195](https://github.com/NatLabRockies/arco/issues/195)) ([5f28bf3](https://github.com/NatLabRockies/arco/commit/5f28bf3894b8a0323a942e40f5ccd202ce2ac3ea))
* **kdl:** add tuple-domain validation and nodal tracer bullet ([#196](https://github.com/NatLabRockies/arco/issues/196)) ([016b3db](https://github.com/NatLabRockies/arco/commit/016b3dbe74712fdf283157e75c5e10a98f7f4c81))
* **kdl:** enable always-on set unpacking ([#216](https://github.com/NatLabRockies/arco/issues/216)) ([e85f232](https://github.com/NatLabRockies/arco/commit/e85f2327177d1733bffcadc5d2f9f9161114bca4))
* **kdl:** explicit tuple subset declarations for constraints ([#192](https://github.com/NatLabRockies/arco/issues/192)) ([b768778](https://github.com/NatLabRockies/arco/commit/b7687783b66f8f4e79c68a4943f3cc60546f171e))
* **kdl:** parse new syntax aliases with compatibility tests ([#182](https://github.com/NatLabRockies/arco/issues/182)) ([493cd8d](https://github.com/NatLabRockies/arco/commit/493cd8d9d3ef9eee9ffc4af3530bb9838e68ec57))
* **kdl:** support entrypoint includes ([#239](https://github.com/NatLabRockies/arco/issues/239)) ([997387a](https://github.com/NatLabRockies/arco/commit/997387ae031e901e495ccc0e26879779cfd31fe3))
* NLP solver integration for related problems solutions ([#238](https://github.com/NatLabRockies/arco/issues/238)) ([6963306](https://github.com/NatLabRockies/arco/commit/6963306782ac965c2a9f4bef228984d0fae5fc12))
* **scip:** embed native solver backend ([#250](https://github.com/NatLabRockies/arco/issues/250)) ([bc2736f](https://github.com/NatLabRockies/arco/commit/bc2736fb6a7392cfc79f12fba70fa0970a6fb45b))
* **solver:** add generic diagnostics ([#247](https://github.com/NatLabRockies/arco/issues/247)) ([f514d4b](https://github.com/NatLabRockies/arco/commit/f514d4bfc8cb959b58e0df4270915a499a1bfff7))
* support generated indexed expression composition ([#267](https://github.com/NatLabRockies/arco/issues/267)) ([59abfa6](https://github.com/NatLabRockies/arco/commit/59abfa6661adce3e38428a79ec34868bbb7c3958))
* **tree-sitter:** add highlight queries for arco_kdl parser ([#131](https://github.com/NatLabRockies/arco/issues/131)) ([0835cca](https://github.com/NatLabRockies/arco/commit/0835cca1b2b371ce5e501f313c3e458a36e258af))
* **xpress:** add CI/dist xpress coverage and path discovery ([#245](https://github.com/NatLabRockies/arco/issues/245)) ([038d4e7](https://github.com/NatLabRockies/arco/commit/038d4e714b0a1195c2c2ed98ca46c0bef91c20b8))
* **Xpress:** Adding xpress support. ([#44](https://github.com/NatLabRockies/arco/issues/44)) ([a26c05d](https://github.com/NatLabRockies/arco/commit/a26c05d68d4688fac810277453e7160efb6e562d))


### Bug Fixes

* adding CI gates  ([#68](https://github.com/NatLabRockies/arco/issues/68)) ([81c298f](https://github.com/NatLabRockies/arco/commit/81c298f87da7cea30b5972597cc2925a528be2f4))
* Adding macos wheels ([#17](https://github.com/NatLabRockies/arco/issues/17)) ([101d19c](https://github.com/NatLabRockies/arco/commit/101d19c4ebd2a38fa0f3b104515271ce98ffa59b))
* Adding missing operators for Variables. ([#19](https://github.com/NatLabRockies/arco/issues/19)) ([f288e2d](https://github.com/NatLabRockies/arco/commit/f288e2d0e62725190b287710d9af94c30adfb24e))
* adding new workflows ([33a1d62](https://github.com/NatLabRockies/arco/commit/33a1d624b7b41a1123bb0e2be7ddc3d816333f75))
* adding some ci guardrails ([3accf06](https://github.com/NatLabRockies/arco/commit/3accf068bf601f2ce2637a73aa8dbb7f114ad77d))
* addressing subset inspection ([#173](https://github.com/NatLabRockies/arco/issues/173)) ([767136c](https://github.com/NatLabRockies/arco/commit/767136c23eb257d77831323e1c904aab381ad1c3))
* **arco-highs:** harden FFI solution extraction error handling ([#29](https://github.com/NatLabRockies/arco/issues/29)) ([9f759a3](https://github.com/NatLabRockies/arco/commit/9f759a3c6daf78ae5b5bb105cf705eb49958eb7c))
* **ci:** add cargo-dist dependency to py-build to avoid tag race ([#86](https://github.com/NatLabRockies/arco/issues/86)) ([46015d9](https://github.com/NatLabRockies/arco/commit/46015d972f1102fd183187fef691c0721ca17dbd))
* **ci:** correct name for the release please config. ([#80](https://github.com/NatLabRockies/arco/issues/80)) ([6a2365d](https://github.com/NatLabRockies/arco/commit/6a2365d37f8b94d29e6a69d0a26c5a4ed14c4601))
* **ci:** dispatch release workflows and fix CLI build quoting ([#59](https://github.com/NatLabRockies/arco/issues/59)) ([9e8838c](https://github.com/NatLabRockies/arco/commit/9e8838cb69fddb530f46c7149f57d13a70b75b49))
* **ci:** isolate cargo-dist CLI release builds from workspace crates ([#61](https://github.com/NatLabRockies/arco/issues/61)) ([5c6ae65](https://github.com/NatLabRockies/arco/commit/5c6ae659c7f1426a79dfa7c74d3ecb3c4e15a065))
* **ci:** set GH_REPO for release gh commands ([dc0a4ce](https://github.com/NatLabRockies/arco/commit/dc0a4ce050048fb5a5a93edc43924425f6a3785f))
* **ci:** stabilize unified python+cli release workflow ([#56](https://github.com/NatLabRockies/arco/issues/56)) ([f22f07c](https://github.com/NatLabRockies/arco/commit/f22f07c8ea441436640fabc9c93d1271364b4565))
* **ci:** trigger release smoke for release-please PRs ([#63](https://github.com/NatLabRockies/arco/issues/63)) ([28b876a](https://github.com/NatLabRockies/arco/commit/28b876ab74479744df7084a55ec065405e31e2b5))
* **ci:** use justfile py-build-ci approach for Python packaging ([#89](https://github.com/NatLabRockies/arco/issues/89)) ([410099e](https://github.com/NatLabRockies/arco/commit/410099e2f08652f8369045fe001053143248c77f))
* cleaning and providing better  examples ([#122](https://github.com/NatLabRockies/arco/issues/122)) ([36973b7](https://github.com/NatLabRockies/arco/commit/36973b7e38074f762bf1d2b90947ab0ce42ca75f))
* **cli:** correct tuple-domain sizes in inspect ([#203](https://github.com/NatLabRockies/arco/issues/203)) ([0e4d0c7](https://github.com/NatLabRockies/arco/commit/0e4d0c7e547dae974a5c22dcc8ed1c61ee19d7b6))
* **cli:** report tuple component sizes in inspect ([#217](https://github.com/NatLabRockies/arco/issues/217)) ([7c14445](https://github.com/NatLabRockies/arco/commit/7c14445870304030dbdda46f0cd4a9c8ee66496a))
* **kdl:** enforce tuple-source validation for rule sets ([#191](https://github.com/NatLabRockies/arco/issues/191)) ([6f872e3](https://github.com/NatLabRockies/arco/commit/6f872e357a82b24ca79e4ddd283883a6df305cbe))
* **kdl:** fail fast on duplicate model declarations ([#219](https://github.com/NatLabRockies/arco/issues/219)) ([10d16c5](https://github.com/NatLabRockies/arco/commit/10d16c56d86fd16e81f7152d0e02f97db8f19ba0))
* **kdl:** replace let-chains for Rust 1.85 compatibility ([#64](https://github.com/NatLabRockies/arco/issues/64)) ([ceb558a](https://github.com/NatLabRockies/arco/commit/ceb558a7624818dae2380c7ac1a6e7eb48306c51))
* key error ([#37](https://github.com/NatLabRockies/arco/issues/37)) ([8304fc3](https://github.com/NatLabRockies/arco/commit/8304fc3bb6e1e0539fb926e59811081a2a196113))
* linter errors not founding the module for type hints ([#15](https://github.com/NatLabRockies/arco/issues/15)) ([019ad25](https://github.com/NatLabRockies/arco/commit/019ad25a55e2358ee69baf7d910b9508be3bbd3d))
* objective function heavy operation ([#27](https://github.com/NatLabRockies/arco/issues/27)) ([9d337e1](https://github.com/NatLabRockies/arco/commit/9d337e1711c31d5211adead79cfff34823819b73))
* preserve tuple index labels in lookups ([#213](https://github.com/NatLabRockies/arco/issues/213)) ([8e6f6b9](https://github.com/NatLabRockies/arco/commit/8e6f6b9b1ca6e8b7c16b1234b82f6864b805da31))
* refresh stale just recipes and stabilize local checks ([#91](https://github.com/NatLabRockies/arco/issues/91)) ([54cbce7](https://github.com/NatLabRockies/arco/commit/54cbce78f1dd06aa57e2299a9002d7384f03f7e9))
* **release:** correct grouped PR title pattern ([#70](https://github.com/NatLabRockies/arco/issues/70)) ([b754244](https://github.com/NatLabRockies/arco/commit/b7542448bc2e59bede5587657a4afe3444aaf7e7))
* **release:** disable draft releases to stop stale release loops ([28c8f7a](https://github.com/NatLabRockies/arco/commit/28c8f7a3eaeab85226373edbc6a3fd51a6333d91))
* **release:** hardcode component in group PR title pattern ([#78](https://github.com/NatLabRockies/arco/issues/78)) ([486b2c7](https://github.com/NatLabRockies/arco/commit/486b2c7b132b81d632ab9c9f66e5593e4989bbee))
* **release:** match tag format to existing tags ([#72](https://github.com/NatLabRockies/arco/issues/72)) ([3a0553c](https://github.com/NatLabRockies/arco/commit/3a0553cbe98ea42a13f87bf1864440297d5d755c))
* **release:** publish GitHub release after artifacts and PyPI ([45bbba4](https://github.com/NatLabRockies/arco/commit/45bbba4ec7a323ffacd60b0ea86be29b5a42cdf6))
* **release:** remove component config to fix release-please parsing ([#84](https://github.com/NatLabRockies/arco/issues/84)) ([c5a7ee4](https://github.com/NatLabRockies/arco/commit/c5a7ee4781b51fd2792c91140446f339cb3390de))
* **release:** remove explicit group-pull-request-title-pattern ([#83](https://github.com/NatLabRockies/arco/issues/83)) ([7389598](https://github.com/NatLabRockies/arco/commit/73895989fb4848698c451a8650f59a4eddf547b5))
* **release:** switch to separate PRs to fix component parsing ([#81](https://github.com/NatLabRockies/arco/issues/81)) ([96b35fd](https://github.com/NatLabRockies/arco/commit/96b35fde135b2bff7ea8b20e619da59f6801da8f))
* removing duplication code path for solver kwargs ([#23](https://github.com/NatLabRockies/arco/issues/23)) ([c96c3f0](https://github.com/NatLabRockies/arco/commit/c96c3f0506ca017643463ef5511cd11adbb5ca50))
* resolve clippy warnings ([#93](https://github.com/NatLabRockies/arco/issues/93)) ([ef2b25c](https://github.com/NatLabRockies/arco/commit/ef2b25c8e551f27e0662fe8411e73d56be727e79))
* resolve prek failures for typos and ruff formatting ([#92](https://github.com/NatLabRockies/arco/issues/92)) ([d0affcc](https://github.com/NatLabRockies/arco/commit/d0affcc27842b0527d61944bf8a2f9c4dcbc1c94))
* support nodal generic indexed lowering and verbose solver logs ([#95](https://github.com/NatLabRockies/arco/issues/95)) ([92bcdbf](https://github.com/NatLabRockies/arco/commit/92bcdbf0b6611954107650eb367dcb89bb484f1f))
* sync kdl overlay with grammar ([#221](https://github.com/NatLabRockies/arco/issues/221)) ([b069c27](https://github.com/NatLabRockies/arco/commit/b069c27b9abbcc53c7c960b50d09935be815c3e5))
* **tree-sitter:** correct highlight queries to match grammar node names ([#133](https://github.com/NatLabRockies/arco/issues/133)) ([30a6438](https://github.com/NatLabRockies/arco/commit/30a64384ae2ba8589ab473678c1bf18434989346))
* **tree-sitter:** simplify arco-kdl multiline strings ([#161](https://github.com/NatLabRockies/arco/issues/161)) ([8f2b527](https://github.com/NatLabRockies/arco/commit/8f2b5275cd6d60309bf8125b594d271229c41674))
* **tree-sitter:** vendor tree-sitter-kdl scanner to remove node_modules dependency ([#132](https://github.com/NatLabRockies/arco/issues/132)) ([6a98cbd](https://github.com/NatLabRockies/arco/commit/6a98cbddb118bffeb569ff8c134db4467f21c49c))
* tuple domain ([#190](https://github.com/NatLabRockies/arco/issues/190)) ([f9b089a](https://github.com/NatLabRockies/arco/commit/f9b089a963e05b07a692ed62782811c3d76dc5ab))
* unified cli releases ([#57](https://github.com/NatLabRockies/arco/issues/57)) ([6cbf8a1](https://github.com/NatLabRockies/arco/commit/6cbf8a1783fb292743f9e45e7db0bab62c2816ad))
* update approach for ipopt integration ([#284](https://github.com/NatLabRockies/arco/issues/284)) ([750d0e3](https://github.com/NatLabRockies/arco/commit/750d0e3b791b0bb07a02ee01c40c35fb0e63f22a))
* **xpress:** update FFI bindings and licensing for Xpress SDK 9+ ([#52](https://github.com/NatLabRockies/arco/issues/52)) ([a4859b3](https://github.com/NatLabRockies/arco/commit/a4859b3017908f6bea8ef2d8b63da7be6a382b63))


### Performance

* **core,highs:** Tier 1 optimizations for 0.4.0 ([#136](https://github.com/NatLabRockies/arco/issues/136)) ([9225df6](https://github.com/NatLabRockies/arco/commit/9225df645938615a4ff9e048b9fe472c2833b3c6))
* **highs:** optimize model-view solve path ([#243](https://github.com/NatLabRockies/arco/issues/243)) ([7789b69](https://github.com/NatLabRockies/arco/commit/7789b697fbff183c40ef5f8ab7237d3e2a2e7ac2))
* improving arco memory management and model built time ([#42](https://github.com/NatLabRockies/arco/issues/42)) ([f034fac](https://github.com/NatLabRockies/arco/commit/f034fac4abf684f53194d800519a769478cb3c4a))
* O(1) metadata lookup with reverse HashMap ([#111](https://github.com/NatLabRockies/arco/issues/111)) ([837ecfc](https://github.com/NatLabRockies/arco/commit/837ecfc3a00573da10ce56da168e90e78e8c1ef1))
* reduce allocations in normalize_terms and export_crs ([#112](https://github.com/NatLabRockies/arco/issues/112)) ([d3260a5](https://github.com/NatLabRockies/arco/commit/d3260a564cb61fcfbc3a29307cd3e3bc94f76757))
* reducing number of copies from highs ([#28](https://github.com/NatLabRockies/arco/issues/28)) ([7ba4dde](https://github.com/NatLabRockies/arco/commit/7ba4dde83d67bf99fbf213ce99e36fa5ed194b07))


### Refactoring

* address code quality audit findings ([#285](https://github.com/NatLabRockies/arco/issues/285)) ([bdda9b2](https://github.com/NatLabRockies/arco/commit/bdda9b22045c600991ab8f012e19010a7bed44b6))
* align examples with single-source data params ([#215](https://github.com/NatLabRockies/arco/issues/215)) ([dd905e7](https://github.com/NatLabRockies/arco/commit/dd905e7af69671bc33d6e62ec617af1240fa5235))
* **arco-bench:** split benchmark runner into modules ([#36](https://github.com/NatLabRockies/arco/issues/36)) ([6e1b891](https://github.com/NatLabRockies/arco/commit/6e1b891b1faf55b6bebeef823945c256c17e0128))
* **arco-blocks:** centralize runtime error logging helper ([#34](https://github.com/NatLabRockies/arco/issues/34)) ([f87e374](https://github.com/NatLabRockies/arco/commit/f87e374118c11649db2aaa3495bc77b1471cc00d))
* **arco-python:** split array wrappers into modules ([#32](https://github.com/NatLabRockies/arco/issues/32)) ([06554a6](https://github.com/NatLabRockies/arco/commit/06554a6aa7106952edfcff26c343ab7fda0e1345))
* **justfile:** improving just file and action/pre-commit ([#97](https://github.com/NatLabRockies/arco/issues/97)) ([58a83a4](https://github.com/NatLabRockies/arco/commit/58a83a4e6b9acd9ee965fd723c46e1ce266a8ee5))
* **kdl:** move inline test KDL into file fixtures ([#220](https://github.com/NatLabRockies/arco/issues/220)) ([6dd1472](https://github.com/NatLabRockies/arco/commit/6dd1472dd99692a7593a21f39288bd0d1075d8c8))
* Reducing memory consumption of CRS matrix and adding more options for creating matrix representations ([#24](https://github.com/NatLabRockies/arco/issues/24)) ([bed359e](https://github.com/NatLabRockies/arco/commit/bed359eb9e8319e646b3e658a4caa6e91b176382))
* removing more slop ([#175](https://github.com/NatLabRockies/arco/issues/175)) ([5140041](https://github.com/NatLabRockies/arco/commit/5140041a6e2f3f03697b8d8c4da37d27e96e60b9))
* stabilizing low-level api ([#118](https://github.com/NatLabRockies/arco/issues/118)) ([de6fc90](https://github.com/NatLabRockies/arco/commit/de6fc90f9c638741dca6ca4d6897fd5e58946991))


### Documentation

* add filter RHS semantics conformance matrix ([#198](https://github.com/NatLabRockies/arco/issues/198)) ([1a43c13](https://github.com/NatLabRockies/arco/commit/1a43c13fd9f40b27b132f205f5aafd353b21a70c))
* add table of contents to arco KDL syntax specification ([#194](https://github.com/NatLabRockies/arco/issues/194)) ([386f2b8](https://github.com/NatLabRockies/arco/commit/386f2b869d508a74afe5232f8c73f1e1764a1536))
* adding full eexplaination of sdom and solver configuration. ([#53](https://github.com/NatLabRockies/arco/issues/53)) ([b382d39](https://github.com/NatLabRockies/arco/commit/b382d39f07e51311691c4ad62e44e4f2dc93083c))
* adding license compliance ([#126](https://github.com/NatLabRockies/arco/issues/126)) ([545eea4](https://github.com/NatLabRockies/arco/commit/545eea4b982318cf013e522013238e3cb628ad10))
* Cleaning stuff around. ([#21](https://github.com/NatLabRockies/arco/issues/21)) ([830430e](https://github.com/NatLabRockies/arco/commit/830430e243152e7c7567494e599d4b3f0c2ec764))
* expand public API rustdoc across crates ([#31](https://github.com/NatLabRockies/arco/issues/31)) ([4bfd6f6](https://github.com/NatLabRockies/arco/commit/4bfd6f6929949d372ea70e0582a43bfe68468c56))
* **readme:** comprehensive README refresh ([#117](https://github.com/NatLabRockies/arco/issues/117)) ([54ba177](https://github.com/NatLabRockies/arco/commit/54ba177b5ba442d47c0fdc2e00ee51ded211a4ae))
* refresh README syntax examples ([#174](https://github.com/NatLabRockies/arco/issues/174)) ([9b0cf57](https://github.com/NatLabRockies/arco/commit/9b0cf5754710f4797e326be12224a532835c4086))
* **release:** define pre-1.0 release lanes ([#272](https://github.com/NatLabRockies/arco/issues/272)) ([01838a9](https://github.com/NatLabRockies/arco/commit/01838a9e8da5c592947b131845119d514b82f328))


### CI

* add benchmark tracking with github-action-benchmark ([#94](https://github.com/NatLabRockies/arco/issues/94)) ([1a2a2d7](https://github.com/NatLabRockies/arco/commit/1a2a2d721b6e01061f64fc1392a3504a5e859aac))
* gate benchmark alerts to pull requests ([#197](https://github.com/NatLabRockies/arco/issues/197)) ([46e102a](https://github.com/NatLabRockies/arco/commit/46e102aa13fef863272d3bdd7a7ba2fa53ebd310))
* **release:** append install instructions and fix repo URLs ([#77](https://github.com/NatLabRockies/arco/issues/77)) ([6109c23](https://github.com/NatLabRockies/arco/commit/6109c2361cb82025d303cc85218312dc0b0ca8ba))
* **release:** revert tag format and fix artifact action SHAs ([#74](https://github.com/NatLabRockies/arco/issues/74)) ([6969f0d](https://github.com/NatLabRockies/arco/commit/6969f0d34a4af10d2faa55bfb372dbf2c2540d81))


### Build

* better ci management ([#38](https://github.com/NatLabRockies/arco/issues/38)) ([0489258](https://github.com/NatLabRockies/arco/commit/0489258f876b938fd74542482af2513eccd8735f))
* **ci:** improving ci ([#67](https://github.com/NatLabRockies/arco/issues/67)) ([a6bc40b](https://github.com/NatLabRockies/arco/commit/a6bc40bcfb95f2abe35632677d588afb5202829a))
* **deps:** bump actions/cache from 5.0.4 to 5.0.5 ([#201](https://github.com/NatLabRockies/arco/issues/201)) ([4d5f9b5](https://github.com/NatLabRockies/arco/commit/4d5f9b5c59144daa588c3eee3b93f62b4055c527))
* **deps:** bump actions/download-artifact from 7.0.0 to 8.0.0 ([#47](https://github.com/NatLabRockies/arco/issues/47)) ([6983021](https://github.com/NatLabRockies/arco/commit/6983021af8ba8b0b1a04f34ce3590a31c166a863))
* **deps:** bump actions/download-artifact from 7.0.0 to 8.0.1 ([#106](https://github.com/NatLabRockies/arco/issues/106)) ([c09984d](https://github.com/NatLabRockies/arco/commit/c09984d4d031fddbd4101fedb86807a74781926e))
* **deps:** bump actions/download-artifact from 8.0.0 to 8.0.1 ([#66](https://github.com/NatLabRockies/arco/issues/66)) ([ae9bf42](https://github.com/NatLabRockies/arco/commit/ae9bf4226cd04ead0c7bf75de40afc0d46be5e99))
* **deps:** bump actions/labeler from 6.0.1 to 6.1.0 ([#249](https://github.com/NatLabRockies/arco/issues/249)) ([008e17e](https://github.com/NatLabRockies/arco/commit/008e17eef83bc985885c8b858e1f5126c8fa8a78))
* **deps:** bump actions/upload-artifact from 6.0.0 to 7.0.0 ([#104](https://github.com/NatLabRockies/arco/issues/104)) ([47799be](https://github.com/NatLabRockies/arco/commit/47799be39a39db6e6f5384fd3b41263b11bca78f))
* **deps:** bump actions/upload-artifact from 6.0.0 to 7.0.0 ([#49](https://github.com/NatLabRockies/arco/issues/49)) ([28c0dd5](https://github.com/NatLabRockies/arco/commit/28c0dd5e30dd0473bdb735989e88c52a505d9538))
* **deps:** bump actions/upload-artifact from 7.0.0 to 7.0.1 ([#200](https://github.com/NatLabRockies/arco/issues/200)) ([a2c9f45](https://github.com/NatLabRockies/arco/commit/a2c9f45f0eaaff414cb4796286c4ed3c2c828932))
* **deps:** bump astral-sh/setup-uv from 7.3.0 to 7.6.0 ([#65](https://github.com/NatLabRockies/arco/issues/65)) ([353f9e5](https://github.com/NatLabRockies/arco/commit/353f9e5949c5b57f1e0cfd6edd3185663b1f34f3))
* **deps:** bump astral-sh/setup-uv from 7.6.0 to 8.0.0 ([#121](https://github.com/NatLabRockies/arco/issues/121)) ([6cb4343](https://github.com/NatLabRockies/arco/commit/6cb43435ae8441e0788e9fc8fe10eef78b6a147a))
* **deps:** bump astral-sh/setup-uv from 8.0.0 to 8.1.0 ([#210](https://github.com/NatLabRockies/arco/issues/210)) ([c846838](https://github.com/NatLabRockies/arco/commit/c8468386d5dea455cec59e08c0b4aadddb0cb16d))
* **deps:** bump benchmark-action/github-action-benchmark ([#120](https://github.com/NatLabRockies/arco/issues/120)) ([a51a951](https://github.com/NatLabRockies/arco/commit/a51a9518463dbaccb39eef95c8eeb535ccf30a66))
* **deps:** bump benchmark-action/github-action-benchmark ([#248](https://github.com/NatLabRockies/arco/issues/248)) ([8116235](https://github.com/NatLabRockies/arco/commit/81162350f9c04f1093c5ea6dd8878d7d98a54636))
* **deps:** bump googleapis/release-please-action from 4.4.0 to 4.4.1 ([#168](https://github.com/NatLabRockies/arco/issues/168)) ([d06890a](https://github.com/NatLabRockies/arco/commit/d06890a982180a0008f4c54d37755bb5c342b53c))
* **deps:** bump googleapis/release-please-action from 4.4.1 to 5.0.0 ([#209](https://github.com/NatLabRockies/arco/issues/209)) ([1234547](https://github.com/NatLabRockies/arco/commit/1234547ef769d6ae2d36bfa5dd1d1080516fbb6d))
* **deps:** bump PyO3/maturin-action from 1.50.0 to 1.50.1 ([#48](https://github.com/NatLabRockies/arco/issues/48)) ([4e498c8](https://github.com/NatLabRockies/arco/commit/4e498c8a7dd935352f621e43b106790c1fcd8d8a))
* **deps:** bump pypa/gh-action-pypi-publish ([#105](https://github.com/NatLabRockies/arco/issues/105)) ([37407eb](https://github.com/NatLabRockies/arco/commit/37407eb6a0014479c445f94415bd8fd8759062d9))
* **deps:** bump pypa/gh-action-pypi-publish from 1.13.0 to 1.14.0 ([#119](https://github.com/NatLabRockies/arco/issues/119)) ([826f5ca](https://github.com/NatLabRockies/arco/commit/826f5ca7b06d206187422d93ddb0a0f354755ed7))


### Chores

* **release:** force 0.1.1 ([6053cbb](https://github.com/NatLabRockies/arco/commit/6053cbb52d837106e2b82eb7cba50c5de76efdb9))


### Tests

* **arco-core:** add direct coverage for types and slack helpers ([#33](https://github.com/NatLabRockies/arco/issues/33)) ([164dcfb](https://github.com/NatLabRockies/arco/commit/164dcfb70fdd94f2cbb98cf263c06790d56b1d05))
* expand filter RHS conformance coverage for mapped and alias cases ([#199](https://github.com/NatLabRockies/arco/issues/199)) ([5d3483f](https://github.com/NatLabRockies/arco/commit/5d3483f1e5de8c3c146f5006de4e6c2a1dd8e702))

## [0.6.1](https://github.com/NatLabRockies/arco/compare/v0.6.0...v0.6.1) (2026-05-16)


### Documentation

* **release:** define pre-1.0 release lanes ([#272](https://github.com/NatLabRockies/arco/issues/272)) ([01838a9](https://github.com/NatLabRockies/arco/commit/01838a9e8da5c592947b131845119d514b82f328))

## [0.6.0](https://github.com/NatLabRockies/arco/compare/v0.5.0...v0.6.0) (2026-05-13)


### Features

* add projection grammar ([#206](https://github.com/NatLabRockies/arco/issues/206)) ([0e30225](https://github.com/NatLabRockies/arco/commit/0e30225ec3caefffe9dfc33ec2640ef3cdf2cc65))
* **cli:** add arco --version output ([#214](https://github.com/NatLabRockies/arco/issues/214)) ([124a403](https://github.com/NatLabRockies/arco/commit/124a4034d4cc8a50dbf39f2b56e3d365408c66c9))
* **cli:** add self-update command ([#251](https://github.com/NatLabRockies/arco/issues/251)) ([8de1e2a](https://github.com/NatLabRockies/arco/commit/8de1e2af8e27c3c22ea06bea0718e9b4f584c948))
* **examples:** add minimal ReEDS KDL benchmark ([#246](https://github.com/NatLabRockies/arco/issues/246)) ([9fa1dde](https://github.com/NatLabRockies/arco/commit/9fa1dde240a4779e89cb2d43334706ed28a2a462))
* **examples:** add ReEDS benchmark ([#244](https://github.com/NatLabRockies/arco/issues/244)) ([f1871f5](https://github.com/NatLabRockies/arco/commit/f1871f5a46636190f0137bae72f9369dfef04404))
* implement solver registry architecture core ([#225](https://github.com/NatLabRockies/arco/issues/225)) ([fa16aa4](https://github.com/NatLabRockies/arco/commit/fa16aa47afbc0d69b74b271a71eac39c4c88c760))
* **kdl:** enable always-on set unpacking ([#216](https://github.com/NatLabRockies/arco/issues/216)) ([e85f232](https://github.com/NatLabRockies/arco/commit/e85f2327177d1733bffcadc5d2f9f9161114bca4))
* **kdl:** support entrypoint includes ([#239](https://github.com/NatLabRockies/arco/issues/239)) ([997387a](https://github.com/NatLabRockies/arco/commit/997387ae031e901e495ccc0e26879779cfd31fe3))
* NLP solver integration for related problems solutions ([#238](https://github.com/NatLabRockies/arco/issues/238)) ([6963306](https://github.com/NatLabRockies/arco/commit/6963306782ac965c2a9f4bef228984d0fae5fc12))
* **scip:** embed native solver backend ([#250](https://github.com/NatLabRockies/arco/issues/250)) ([bc2736f](https://github.com/NatLabRockies/arco/commit/bc2736fb6a7392cfc79f12fba70fa0970a6fb45b))
* **solver:** add generic diagnostics ([#247](https://github.com/NatLabRockies/arco/issues/247)) ([f514d4b](https://github.com/NatLabRockies/arco/commit/f514d4bfc8cb959b58e0df4270915a499a1bfff7))
* **xpress:** add CI/dist xpress coverage and path discovery ([#245](https://github.com/NatLabRockies/arco/issues/245)) ([038d4e7](https://github.com/NatLabRockies/arco/commit/038d4e714b0a1195c2c2ed98ca46c0bef91c20b8))


### Bug Fixes

* **cli:** report tuple component sizes in inspect ([#217](https://github.com/NatLabRockies/arco/issues/217)) ([7c14445](https://github.com/NatLabRockies/arco/commit/7c14445870304030dbdda46f0cd4a9c8ee66496a))
* **kdl:** fail fast on duplicate model declarations ([#219](https://github.com/NatLabRockies/arco/issues/219)) ([10d16c5](https://github.com/NatLabRockies/arco/commit/10d16c56d86fd16e81f7152d0e02f97db8f19ba0))
* preserve tuple index labels in lookups ([#213](https://github.com/NatLabRockies/arco/issues/213)) ([8e6f6b9](https://github.com/NatLabRockies/arco/commit/8e6f6b9b1ca6e8b7c16b1234b82f6864b805da31))
* sync kdl overlay with grammar ([#221](https://github.com/NatLabRockies/arco/issues/221)) ([b069c27](https://github.com/NatLabRockies/arco/commit/b069c27b9abbcc53c7c960b50d09935be815c3e5))


### Performance

* **highs:** optimize model-view solve path ([#243](https://github.com/NatLabRockies/arco/issues/243)) ([7789b69](https://github.com/NatLabRockies/arco/commit/7789b697fbff183c40ef5f8ab7237d3e2a2e7ac2))


### Refactoring

* align examples with single-source data params ([#215](https://github.com/NatLabRockies/arco/issues/215)) ([dd905e7](https://github.com/NatLabRockies/arco/commit/dd905e7af69671bc33d6e62ec617af1240fa5235))
* **kdl:** move inline test KDL into file fixtures ([#220](https://github.com/NatLabRockies/arco/issues/220)) ([6dd1472](https://github.com/NatLabRockies/arco/commit/6dd1472dd99692a7593a21f39288bd0d1075d8c8))


### Build

* **deps:** bump actions/labeler from 6.0.1 to 6.1.0 ([#249](https://github.com/NatLabRockies/arco/issues/249)) ([008e17e](https://github.com/NatLabRockies/arco/commit/008e17eef83bc985885c8b858e1f5126c8fa8a78))
* **deps:** bump astral-sh/setup-uv from 8.0.0 to 8.1.0 ([#210](https://github.com/NatLabRockies/arco/issues/210)) ([c846838](https://github.com/NatLabRockies/arco/commit/c8468386d5dea455cec59e08c0b4aadddb0cb16d))
* **deps:** bump benchmark-action/github-action-benchmark ([#248](https://github.com/NatLabRockies/arco/issues/248)) ([8116235](https://github.com/NatLabRockies/arco/commit/81162350f9c04f1093c5ea6dd8878d7d98a54636))
* **deps:** bump googleapis/release-please-action from 4.4.1 to 5.0.0 ([#209](https://github.com/NatLabRockies/arco/issues/209)) ([1234547](https://github.com/NatLabRockies/arco/commit/1234547ef769d6ae2d36bfa5dd1d1080516fbb6d))

## [0.5.0](https://github.com/NatLabRockies/arco/compare/v0.4.0...v0.5.0) (2026-04-24)


### Features

* **bench:** resource-monitor integration for benchmark orchestration ([#159](https://github.com/NatLabRockies/arco/issues/159)) ([#160](https://github.com/NatLabRockies/arco/issues/160)) ([69d05b0](https://github.com/NatLabRockies/arco/commit/69d05b0d56649150ebd96d91c254576a31dadbc3))
* **kdl:** add scoped inferred IDs for tuple diagnostics ([#195](https://github.com/NatLabRockies/arco/issues/195)) ([5f28bf3](https://github.com/NatLabRockies/arco/commit/5f28bf3894b8a0323a942e40f5ccd202ce2ac3ea))
* **kdl:** add tuple-domain validation and nodal tracer bullet ([#196](https://github.com/NatLabRockies/arco/issues/196)) ([016b3db](https://github.com/NatLabRockies/arco/commit/016b3dbe74712fdf283157e75c5e10a98f7f4c81))
* **kdl:** explicit tuple subset declarations for constraints ([#192](https://github.com/NatLabRockies/arco/issues/192)) ([b768778](https://github.com/NatLabRockies/arco/commit/b7687783b66f8f4e79c68a4943f3cc60546f171e))
* **kdl:** parse new syntax aliases with compatibility tests ([#182](https://github.com/NatLabRockies/arco/issues/182)) ([493cd8d](https://github.com/NatLabRockies/arco/commit/493cd8d9d3ef9eee9ffc4af3530bb9838e68ec57))


### Bug Fixes

* addressing subset inspection ([#173](https://github.com/NatLabRockies/arco/issues/173)) ([767136c](https://github.com/NatLabRockies/arco/commit/767136c23eb257d77831323e1c904aab381ad1c3))
* **cli:** correct tuple-domain sizes in inspect ([#203](https://github.com/NatLabRockies/arco/issues/203)) ([0e4d0c7](https://github.com/NatLabRockies/arco/commit/0e4d0c7e547dae974a5c22dcc8ed1c61ee19d7b6))
* **kdl:** enforce tuple-source validation for rule sets ([#191](https://github.com/NatLabRockies/arco/issues/191)) ([6f872e3](https://github.com/NatLabRockies/arco/commit/6f872e357a82b24ca79e4ddd283883a6df305cbe))
* **tree-sitter:** simplify arco-kdl multiline strings ([#161](https://github.com/NatLabRockies/arco/issues/161)) ([8f2b527](https://github.com/NatLabRockies/arco/commit/8f2b5275cd6d60309bf8125b594d271229c41674))
* tuple domain ([#190](https://github.com/NatLabRockies/arco/issues/190)) ([f9b089a](https://github.com/NatLabRockies/arco/commit/f9b089a963e05b07a692ed62782811c3d76dc5ab))


### Refactoring

* removing more slop ([#175](https://github.com/NatLabRockies/arco/issues/175)) ([5140041](https://github.com/NatLabRockies/arco/commit/5140041a6e2f3f03697b8d8c4da37d27e96e60b9))


### Documentation

* add filter RHS semantics conformance matrix ([#198](https://github.com/NatLabRockies/arco/issues/198)) ([1a43c13](https://github.com/NatLabRockies/arco/commit/1a43c13fd9f40b27b132f205f5aafd353b21a70c))
* add table of contents to arco KDL syntax specification ([#194](https://github.com/NatLabRockies/arco/issues/194)) ([386f2b8](https://github.com/NatLabRockies/arco/commit/386f2b869d508a74afe5232f8c73f1e1764a1536))
* refresh README syntax examples ([#174](https://github.com/NatLabRockies/arco/issues/174)) ([9b0cf57](https://github.com/NatLabRockies/arco/commit/9b0cf5754710f4797e326be12224a532835c4086))


### CI

* gate benchmark alerts to pull requests ([#197](https://github.com/NatLabRockies/arco/issues/197)) ([46e102a](https://github.com/NatLabRockies/arco/commit/46e102aa13fef863272d3bdd7a7ba2fa53ebd310))


### Build

* **deps:** bump actions/cache from 5.0.4 to 5.0.5 ([#201](https://github.com/NatLabRockies/arco/issues/201)) ([4d5f9b5](https://github.com/NatLabRockies/arco/commit/4d5f9b5c59144daa588c3eee3b93f62b4055c527))
* **deps:** bump actions/upload-artifact from 7.0.0 to 7.0.1 ([#200](https://github.com/NatLabRockies/arco/issues/200)) ([a2c9f45](https://github.com/NatLabRockies/arco/commit/a2c9f45f0eaaff414cb4796286c4ed3c2c828932))
* **deps:** bump googleapis/release-please-action from 4.4.0 to 4.4.1 ([#168](https://github.com/NatLabRockies/arco/issues/168)) ([d06890a](https://github.com/NatLabRockies/arco/commit/d06890a982180a0008f4c54d37755bb5c342b53c))


### Tests

* expand filter RHS conformance coverage for mapped and alias cases ([#199](https://github.com/NatLabRockies/arco/issues/199)) ([5d3483f](https://github.com/NatLabRockies/arco/commit/5d3483f1e5de8c3c146f5006de4e6c2a1dd8e702))

## [0.4.0](https://github.com/NatLabRockies/arco/compare/v0.3.0...v0.4.0) (2026-04-13)


### Features

* **tree-sitter:** add highlight queries for arco_kdl parser ([#131](https://github.com/NatLabRockies/arco/issues/131)) ([0835cca](https://github.com/NatLabRockies/arco/commit/0835cca1b2b371ce5e501f313c3e458a36e258af))


### Bug Fixes

* **tree-sitter:** correct highlight queries to match grammar node names ([#133](https://github.com/NatLabRockies/arco/issues/133)) ([30a6438](https://github.com/NatLabRockies/arco/commit/30a64384ae2ba8589ab473678c1bf18434989346))
* **tree-sitter:** vendor tree-sitter-kdl scanner to remove node_modules dependency ([#132](https://github.com/NatLabRockies/arco/issues/132)) ([6a98cbd](https://github.com/NatLabRockies/arco/commit/6a98cbddb118bffeb569ff8c134db4467f21c49c))


### Performance

* **core,highs:** Tier 1 optimizations for 0.4.0 ([#136](https://github.com/NatLabRockies/arco/issues/136)) ([9225df6](https://github.com/NatLabRockies/arco/commit/9225df645938615a4ff9e048b9fe472c2833b3c6))


### Documentation

* adding license compliance ([#126](https://github.com/NatLabRockies/arco/issues/126)) ([545eea4](https://github.com/NatLabRockies/arco/commit/545eea4b982318cf013e522013238e3cb628ad10))

## [0.3.0](https://github.com/NatLabRockies/arco/compare/v0.2.8...v0.3.0) (2026-04-08)


### Features

* adding pretty printing of sets/variables ([#102](https://github.com/NatLabRockies/arco/issues/102)) ([d56cc33](https://github.com/NatLabRockies/arco/commit/d56cc335b73c384f37f42922ec55f476c2be02f4))
* expose constraint duals via unified report keyword ([#103](https://github.com/NatLabRockies/arco/issues/103)) ([bdfa40b](https://github.com/NatLabRockies/arco/commit/bdfa40b414e2f23e81498093217100d0674fe785))


### Bug Fixes

* **ci:** use justfile py-build-ci approach for Python packaging ([#89](https://github.com/NatLabRockies/arco/issues/89)) ([410099e](https://github.com/NatLabRockies/arco/commit/410099e2f08652f8369045fe001053143248c77f))
* cleaning and providing better  examples ([#122](https://github.com/NatLabRockies/arco/issues/122)) ([36973b7](https://github.com/NatLabRockies/arco/commit/36973b7e38074f762bf1d2b90947ab0ce42ca75f))
* refresh stale just recipes and stabilize local checks ([#91](https://github.com/NatLabRockies/arco/issues/91)) ([54cbce7](https://github.com/NatLabRockies/arco/commit/54cbce78f1dd06aa57e2299a9002d7384f03f7e9))
* resolve clippy warnings ([#93](https://github.com/NatLabRockies/arco/issues/93)) ([ef2b25c](https://github.com/NatLabRockies/arco/commit/ef2b25c8e551f27e0662fe8411e73d56be727e79))
* resolve prek failures for typos and ruff formatting ([#92](https://github.com/NatLabRockies/arco/issues/92)) ([d0affcc](https://github.com/NatLabRockies/arco/commit/d0affcc27842b0527d61944bf8a2f9c4dcbc1c94))
* support nodal generic indexed lowering and verbose solver logs ([#95](https://github.com/NatLabRockies/arco/issues/95)) ([92bcdbf](https://github.com/NatLabRockies/arco/commit/92bcdbf0b6611954107650eb367dcb89bb484f1f))


### Performance

* O(1) metadata lookup with reverse HashMap ([#111](https://github.com/NatLabRockies/arco/issues/111)) ([837ecfc](https://github.com/NatLabRockies/arco/commit/837ecfc3a00573da10ce56da168e90e78e8c1ef1))
* reduce allocations in normalize_terms and export_crs ([#112](https://github.com/NatLabRockies/arco/issues/112)) ([d3260a5](https://github.com/NatLabRockies/arco/commit/d3260a564cb61fcfbc3a29307cd3e3bc94f76757))


### Refactoring

* **justfile:** improving just file and action/pre-commit ([#97](https://github.com/NatLabRockies/arco/issues/97)) ([58a83a4](https://github.com/NatLabRockies/arco/commit/58a83a4e6b9acd9ee965fd723c46e1ce266a8ee5))
* stabilizing low-level api ([#118](https://github.com/NatLabRockies/arco/issues/118)) ([de6fc90](https://github.com/NatLabRockies/arco/commit/de6fc90f9c638741dca6ca4d6897fd5e58946991))


### Documentation

* **readme:** comprehensive README refresh ([#117](https://github.com/NatLabRockies/arco/issues/117)) ([54ba177](https://github.com/NatLabRockies/arco/commit/54ba177b5ba442d47c0fdc2e00ee51ded211a4ae))


### CI

* add benchmark tracking with github-action-benchmark ([#94](https://github.com/NatLabRockies/arco/issues/94)) ([1a2a2d7](https://github.com/NatLabRockies/arco/commit/1a2a2d721b6e01061f64fc1392a3504a5e859aac))


### Build

* **deps:** bump actions/download-artifact from 7.0.0 to 8.0.1 ([#106](https://github.com/NatLabRockies/arco/issues/106)) ([c09984d](https://github.com/NatLabRockies/arco/commit/c09984d4d031fddbd4101fedb86807a74781926e))
* **deps:** bump actions/upload-artifact from 6.0.0 to 7.0.0 ([#104](https://github.com/NatLabRockies/arco/issues/104)) ([47799be](https://github.com/NatLabRockies/arco/commit/47799be39a39db6e6f5384fd3b41263b11bca78f))
* **deps:** bump astral-sh/setup-uv from 7.6.0 to 8.0.0 ([#121](https://github.com/NatLabRockies/arco/issues/121)) ([6cb4343](https://github.com/NatLabRockies/arco/commit/6cb43435ae8441e0788e9fc8fe10eef78b6a147a))
* **deps:** bump benchmark-action/github-action-benchmark ([#120](https://github.com/NatLabRockies/arco/issues/120)) ([a51a951](https://github.com/NatLabRockies/arco/commit/a51a9518463dbaccb39eef95c8eeb535ccf30a66))
* **deps:** bump pypa/gh-action-pypi-publish ([#105](https://github.com/NatLabRockies/arco/issues/105)) ([37407eb](https://github.com/NatLabRockies/arco/commit/37407eb6a0014479c445f94415bd8fd8759062d9))
* **deps:** bump pypa/gh-action-pypi-publish from 1.13.0 to 1.14.0 ([#119](https://github.com/NatLabRockies/arco/issues/119)) ([826f5ca](https://github.com/NatLabRockies/arco/commit/826f5ca7b06d206187422d93ddb0a0f354755ed7))

## [0.2.8](https://github.com/NatLabRockies/arco/compare/v0.2.7...v0.2.8) (2026-03-27)


### Bug Fixes

* **ci:** add cargo-dist dependency to py-build to avoid tag race ([#86](https://github.com/NatLabRockies/arco/issues/86)) ([46015d9](https://github.com/NatLabRockies/arco/commit/46015d972f1102fd183187fef691c0721ca17dbd))
* **ci:** correct name for the release please config. ([#80](https://github.com/NatLabRockies/arco/issues/80)) ([6a2365d](https://github.com/NatLabRockies/arco/commit/6a2365d37f8b94d29e6a69d0a26c5a4ed14c4601))
* **release:** remove component config to fix release-please parsing ([#84](https://github.com/NatLabRockies/arco/issues/84)) ([c5a7ee4](https://github.com/NatLabRockies/arco/commit/c5a7ee4781b51fd2792c91140446f339cb3390de))
* **release:** remove explicit group-pull-request-title-pattern ([#83](https://github.com/NatLabRockies/arco/issues/83)) ([7389598](https://github.com/NatLabRockies/arco/commit/73895989fb4848698c451a8650f59a4eddf547b5))
* **release:** switch to separate PRs to fix component parsing ([#81](https://github.com/NatLabRockies/arco/issues/81)) ([96b35fd](https://github.com/NatLabRockies/arco/commit/96b35fde135b2bff7ea8b20e619da59f6801da8f))

## [0.2.7](https://github.com/NatLabRockies/arco/compare/v0.2.6...v0.2.7) (2026-03-27)


### Bug Fixes

* **release:** hardcode component in group PR title pattern ([#78](https://github.com/NatLabRockies/arco/issues/78)) ([486b2c7](https://github.com/NatLabRockies/arco/commit/486b2c7b132b81d632ab9c9f66e5593e4989bbee))

## [0.2.6](https://github.com/NatLabRockies/arco/compare/v0.2.5...v0.2.6) (2026-03-27)


### CI

* **release:** append install instructions and fix repo URLs ([#77](https://github.com/NatLabRockies/arco/issues/77)) ([6109c23](https://github.com/NatLabRockies/arco/commit/6109c2361cb82025d303cc85218312dc0b0ca8ba))


### Build

* **deps:** bump actions/download-artifact from 8.0.0 to 8.0.1 ([#66](https://github.com/NatLabRockies/arco/issues/66)) ([ae9bf42](https://github.com/NatLabRockies/arco/commit/ae9bf4226cd04ead0c7bf75de40afc0d46be5e99))
* **deps:** bump astral-sh/setup-uv from 7.3.0 to 7.6.0 ([#65](https://github.com/NatLabRockies/arco/issues/65)) ([353f9e5](https://github.com/NatLabRockies/arco/commit/353f9e5949c5b57f1e0cfd6edd3185663b1f34f3))

## [0.2.5](https://github.com/NatLabRockies/arco/compare/v0.2.5...v0.2.5) (2026-03-27)


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


### CI

* **release:** revert tag format and fix artifact action SHAs ([#74](https://github.com/NatLabRockies/arco/issues/74)) ([6969f0d](https://github.com/NatLabRockies/arco/commit/6969f0d34a4af10d2faa55bfb372dbf2c2540d81))


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
