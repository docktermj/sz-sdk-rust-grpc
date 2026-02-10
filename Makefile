# Makefile for sz-sdk-rust-grpc

# Detect the operating system and architecture.

include makefiles/osdetect.mk

# -----------------------------------------------------------------------------
# Variables
# -----------------------------------------------------------------------------

PROGRAM_NAME := $(shell basename `git rev-parse --show-toplevel`)
MAKEFILE_PATH := $(abspath $(firstword $(MAKEFILE_LIST)))
MAKEFILE_DIRECTORY := $(shell dirname $(MAKEFILE_PATH))
BUILD_VERSION := $(shell git describe --always --tags --abbrev=0 --dirty 2>/dev/null || echo "0.0.0")

.EXPORT_ALL_VARIABLES:

# -----------------------------------------------------------------------------
# The first "make" target runs as default.
# -----------------------------------------------------------------------------

.PHONY: default
default: help

# -----------------------------------------------------------------------------
# Operating System / Architecture targets
# -----------------------------------------------------------------------------

# -include makefiles/$(OSTYPE).mk
# -include makefiles/$(OSTYPE)_$(OSARCH).mk

# -----------------------------------------------------------------------------
# Dependency management
# -----------------------------------------------------------------------------

.PHONY: dependencies
dependencies:
	@cargo update

# -----------------------------------------------------------------------------
# Setup - start a Senzing gRPC server for testing
# -----------------------------------------------------------------------------

.PHONY: setup
setup:
	@docker run \
		--detach \
		--env SENZING_TOOLS_ENABLE_ALL=true \
		--name senzing-serve-grpc \
		--publish 8261:8261 \
		--rm \
		senzing/serve-grpc
	$(info senzing/serve-grpc running in background.)
	$(info Sleeping to allow gRPC server to come up.)
	@sleep 3

.PHONY: setup-mutual-tls
setup-mutual-tls:
	@docker run \
		--detach \
		--env SENZING_TOOLS_CLIENT_CA_CERTIFICATE_FILE=/testdata/certificates/certificate-authority/certificate.pem \
		--env SENZING_TOOLS_ENABLE_ALL=true \
		--env SENZING_TOOLS_SERVER_CERTIFICATE_FILE=/testdata/certificates/server/certificate.pem \
		--env SENZING_TOOLS_SERVER_KEY_FILE=/testdata/certificates/server/private_key.pem \
		--name senzing-serve-grpc \
		--publish 8261:8261 \
		--rm \
		--volume $(MAKEFILE_DIRECTORY)/testdata:/testdata \
		senzing/serve-grpc
	$(info senzing/serve-grpc with Mutual TLS running in background.)
	$(info Sleeping to allow gRPC server to come up.)
	@sleep 3

.PHONY: setup-server-side-tls
setup-server-side-tls:
	@docker run \
		--detach \
		--env SENZING_TOOLS_ENABLE_ALL=true \
		--env SENZING_TOOLS_SERVER_CERTIFICATE_FILE=/testdata/certificates/server/certificate.pem \
		--env SENZING_TOOLS_SERVER_KEY_FILE=/testdata/certificates/server/private_key.pem \
		--name senzing-serve-grpc \
		--publish 8261:8261 \
		--rm \
		--volume $(MAKEFILE_DIRECTORY)/testdata:/testdata \
		senzing/serve-grpc
	$(info senzing/serve-grpc with Server-Side TLS running in background.)
	$(info Sleeping to allow gRPC server to come up.)
	@sleep 3

# -----------------------------------------------------------------------------
# Lint
# -----------------------------------------------------------------------------

.PHONY: lint
lint: cspell
	@cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	@cargo fmt

.PHONY: fmt-check
fmt-check:
	@cargo fmt -- --check

# -----------------------------------------------------------------------------
# Build
# -----------------------------------------------------------------------------

.PHONY: build
build:
	@cargo build

# -----------------------------------------------------------------------------
# Run
# -----------------------------------------------------------------------------

# .PHONY: run
# run: run-osarch-specific

# -----------------------------------------------------------------------------
# Test
# -----------------------------------------------------------------------------

.PHONY: test
test:
	@cargo test -- --show-output

# -----------------------------------------------------------------------------
# Coverage
# -----------------------------------------------------------------------------

# .PHONY: coverage
# coverage: coverage-osarch-specific

# -----------------------------------------------------------------------------
# Documentation
# -----------------------------------------------------------------------------

# .PHONY: documentation
# documentation: documentation-osarch-specific

# -----------------------------------------------------------------------------
# Clean
# -----------------------------------------------------------------------------

.PHONY: clean
clean:
	@docker rm --force senzing-serve-grpc 2>/dev/null || true

# -----------------------------------------------------------------------------
# Utility targets
# -----------------------------------------------------------------------------

.PHONY: help
help:
	$(info Build $(PROGRAM_NAME) version $(BUILD_VERSION))
	$(info Makefile targets:)
	@$(MAKE) -pRrq -f $(firstword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$$@$$' | xargs

.PHONY: print-make-variables
print-make-variables:
	@$(foreach V,$(sort $(.VARIABLES)), \
		$(if $(filter-out environment% default automatic, \
		$(origin $V)),$(info $V=$($V) ($(value $V)))))

# -----------------------------------------------------------------------------
# Specific programs
# -----------------------------------------------------------------------------

.PHONY: cspell
cspell:
	@cspell lint --dot .
