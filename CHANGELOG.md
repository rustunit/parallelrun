# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.3.2] - 2024-10-04

### Fixes
* Fix Windows build

## [0.3.1] - 2024-10-04

### Fixes
* Do not register for SIGKILL prevent a crash

## [0.3.0] - 2024-10-04

### Added
* Signals sent to parallelrun will be forwarded to child processes (supports SIGTERM, SIGINT, SIGHUP, SIGKILL, SIGQUIT)

## [0.2.0] - 2024-09-26

### Added
* Add windows support and ci