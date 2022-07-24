Checks a URL to make sure there's no broken links.

# Usage

```sh
./linkchecker http://url-to-check.com
```

If all links are HTTP 200, exits silently with code 0. If a broken link is found, prints one link per line and exits with code 1.

# Developer notes

Concurrent stream processing is still a pain in the ass in Rust. I never know when to use `then` or `map`. I usually spend 10 minutes trying to figure out where to put the `.buffer_unordered` call.

Doing the calls in serial takes around 5.7 seconds for a sample blog post. Parallel checking only takes 1.3 seconds. Currently most of that time is spent doing TLS handshakes to the various hosts. Tested on my work machine, an Intel Macbook Pro. 