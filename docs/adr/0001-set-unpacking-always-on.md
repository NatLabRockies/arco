# Always-on set unpacking semantics for `index <set>`

Arco will treat `index <set_name>` as set unpacking semantics in all supported locations (`constraint`, `control`, and `param`) without an opt-in compatibility flag. This keeps authoring simple and consistent across tuple and non-tuple sets, and avoids dual-language behavior that would fragment docs and examples. Existing models that relied on scalar-only interpretation for tuple sets must migrate to the explicit alias form (`index <var> { in <set> }`) where needed.
