docs:
	RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo +nightly doc --no-deps
	cat website/content/configuration.tmpl > website/content/configuration.md
	cargo run -p docgen target/doc/retherm.json \
	   AwayConfig BackplateConfig HomeAssistantConfig BacklightConfig ScheduleConfig \
	   >>website/content/configuration.md
