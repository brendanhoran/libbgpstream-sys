# libbgpstream-sys
Rust system bindings for bgpstream


## Why did you roll your own tarball, when a release tarball is provided?
Since the codebase has issues building on a modern toolchain(1).   
We need to patch the configure script inputs(.ac) file to fix these issues.   
This requires us to run "autogen.sh" to create the configure script.   
"autogen.sh" is not supplied in the release tarball.   


(1) https://github.com/CAIDA/libbgpstream/issues/227   

## System dependencies

**Your system must have libcurl installed!**    

I'd like to use the static version of `curl-sys` but that has caused other issues.   

Further details on what this issue looks like can be found here:   
https://github.com/brendanhoran/wandio-sys#system-dependencies   

This results in linking to the following system libraries in this crates [buils.rs](https://github.com/brendanhoran/libbgpstream-sys/blob/4325a413b706edd94ca4d10857d845d6252c46bb/build.rs#L107).   
