$plugins = @(
	"depth"
	"simple-channel"
	"simple-clipper"
	"simple-filter"
	"simple-gain"
	"sundara-eq"
)

foreach($plugin in $plugins) {
	cargo run --bin bundler -- bundle $plugin $args
	if($LastExitCode -ne 0) {
		exit 1
	}
}
