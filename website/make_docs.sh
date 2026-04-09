#!/bin/bash

#
# Generate website source markdown documents from rust struct comments
#

cat content/configuration.tmpl > content/configuration.md
cat content/theme.tmpl > content/theme.md

(cd ..
   # At this time, json output is an unstable feature; requires nightly tools
   RUSTDOCFLAGS="-Z unstable-options --output-format json" \
      cargo +nightly doc --no-deps

   cargo run -p docgen target/doc/retherm.json \
      AwayConfig BackplateConfig HomeAssistantConfig BacklightConfig ScheduleConfig \
      >>website/content/configuration.md

   # cargo run -p docgen target/doc/retherm.json \
   #    ThemeConfig \
   #    >>website/content/theme.md
)
