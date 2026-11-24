# mp4parse-rust threat model

Firefox uses `mp4parse_capi` to read metadata from MP4 and AVIF files.
Web content can supply the entire file. Treat its bytes, including container
sizes, offsets, counts, and metadata, as untrusted.

## Scope

- Review the Rust MP4 and AVIF parsers (`read_mp4` and `read_avif`) and the C API
  (`mp4parse_new` and `mp4parse_avif_new`), including how the C API exposes
  parsed metadata and sample tables to its callers.
- Include the default configuration and `missing-pixi-permitted`, which Firefox
  enables. Treat C callers and their callbacks as trusted and assume they meet
  the documented pointer and lifetime contracts.
- Firefox integration code, downstream decoders, and the test fixture
  submodules are outside this repository's review scope.

## Security impact

Prioritize memory safety and violations of the C API's documented guarantees
that malicious media can trigger. Consider crashes and disproportionate CPU or
memory use when they affect a realistic consumer. Rejecting malformed media
with a parse error is expected behavior.
