.PHONY: build install test clean

PREFIX ?= $(HOME)/.qntx
QNTX_ROOT ?= $(HOME)/SBVH/teranos/QNTX
PLUGIN = sbvhAPI

# Auto-bump the patch before every build. Any edit to a plugin must change the
# version — the number reported in the UI and the log is the only proof that
# new code is running. Doing it here means it cannot be forgotten.
#
# Read via cargo metadata and write by exact line match: no pattern matching
# anywhere, so a version string can never be mangled by a stray character.
define bump-version
	@CURRENT=$$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'); \
	 MAJOR=$$(echo $$CURRENT | cut -d. -f1); \
	 MINOR=$$(echo $$CURRENT | cut -d. -f2); \
	 PATCH=$$(echo $$CURRENT | cut -d. -f3); \
	 NEXT="$$MAJOR.$$MINOR.$$((PATCH + 1))"; \
	 awk -v old="version = \"$$CURRENT\"" -v new="version = \"$$NEXT\"" \
	     '$$0==old{print new; next} {print}' Cargo.toml > Cargo.toml.next; \
	 mv Cargo.toml.next Cargo.toml; \
	 echo "  ⬆ $$CURRENT → $$NEXT"
endef

# Tell the running QNTX to hot-restart, then poll until it reports the version
# we just built. A restart that silently kept the old binary is the failure
# this exists to catch, so a mismatch is an error rather than a warning.
define restart-plugin
	@TOML_PORT=$$(grep '^port' $(QNTX_ROOT)/am.toml 2>/dev/null | head -1 | cut -d= -f2 | tr -dc '0-9'); \
	 PORT=$${BACKEND_PORT:-$${TOML_PORT:-8770}}; \
	 EXPECTED=$$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'); \
	 curl -sf -X POST http://127.0.0.1:$$PORT/api/plugins/$(PLUGIN)/restart > /dev/null 2>&1 || { \
	   echo "  ⊘ NOT LOADED — QNTX is not running on $$PORT; binary is built but not active"; \
	   exit 0; }; \
	 echo "  ↻ RESTARTING $(PLUGIN) (expecting $$EXPECTED)..."; \
	 for i in $$(seq 1 60); do \
	   STATE=$$(curl -sf http://127.0.0.1:$$PORT/api/plugins 2>/dev/null \
	     | jq -r '.plugins[]? | select(.name=="$(PLUGIN)") | .state'); \
	   if [ "$$STATE" = "running" ]; then \
	     VERSION=$$(curl -sf http://127.0.0.1:$$PORT/api/plugins 2>/dev/null \
	       | jq -r '.plugins[]? | select(.name=="$(PLUGIN)") | .version'); \
	     if [ -z "$$VERSION" ]; then :; \
	     elif [ "$$VERSION" = "$$EXPECTED" ]; then \
	       echo "  ✓ LOADED — $(PLUGIN) $$VERSION live at http://127.0.0.1:$$PORT"; exit 0; \
	     else \
	       echo "  ✗ WRONG VERSION — expected $$EXPECTED, got $$VERSION"; exit 1; \
	     fi; \
	   fi; \
	   sleep 1; \
	 done; \
	 echo "  ✗ $(PLUGIN) did not reach running state within 60s (expected $$EXPECTED)"; exit 1
endef

install: ## Build, install, and hot-restart the plugin
	$(call bump-version)
	@echo "Building $(PLUGIN)..."
	@cargo build --release
	@mkdir -p $(PREFIX)/plugins
	@cp target/release/$(PLUGIN) $(PREFIX)/plugins/$(PLUGIN)
	@chmod +x $(PREFIX)/plugins/$(PLUGIN)
	@echo "✓ BUILD OK — installed to $(PREFIX)/plugins/$(PLUGIN)"
	$(call restart-plugin)

build: ## Build without installing
	@cargo build --release

test: ## Unit tests
	@cargo test

clean: ## Remove build artifacts
	@cargo clean
