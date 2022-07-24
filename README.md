Checks a URL to make sure there's no broken links.

# Usage

```sh
./linkchecker http://url-to-check.com
```

If all links are HTTP 200, exits silently with code 0. If a broken link is found, prints one link per line and exits with code 1.

# Developer notes

Concurrent stream processing is still a pain in the ass in Rust. I never know when to use `then` or `map`. I usually spend 10 minutes trying to figure out where to put the `.buffer_unordered` call.

Flamegraphs show that the program spends almost all its CPU time doing TLS handshakes to the various hosts. You can run benchmarks with `./benchmarks` (requires [hyperfine](https://github.com/sharkdp/hyperfine)) -- PRs to improve performance are _very_ welcome. On my work machine, an Intel Macbook Pro, takes around 1 second to check my latest blog post.
