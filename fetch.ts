import { exists } from "jsr:@std/fs";
import { z } from "https://deno.land/x/zod@v3.23.8/mod.ts";
import $ from "https://deno.land/x/dax@0.39.2/mod.ts";

if (!(await exists("./output"))) {
    // Note: this requires that git-lfs is installed.
    // From https://stackoverflow.com/a/60729017/7589775
    await $`git clone --no-checkout --depth=1 https://github.com/oeis/oeisdata.git ./output`;
    await $`git sparse-checkout init --cone`.cwd("./output");
    await $`git sparse-checkout set seq`.cwd("./output")
    await $`git checkout`.cwd("./output")
}
