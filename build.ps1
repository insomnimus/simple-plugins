$plugins = @(
	"simple-clipper"
	"simple-filter"
	"simple-gain"
)

foreach($plugin in $plugins) {
	cargo run --bin bundler -- bundle $plugin $args
	if($LastExitCode -ne 0) {
		exit 1
	}
}
