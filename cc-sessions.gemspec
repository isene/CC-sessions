# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = "cc-sessions"
  spec.version       = "1.4.1"
  spec.authors       = ["Geir Isene"]
  spec.email         = ["g@isene.com"]

  spec.summary       = "Bookmark and resume Claude Code sessions with tags"
  spec.description   = "A simple tool for bookmarking and resuming Claude Code sessions. " \
                       "Tag sessions with meaningful names using /bm inside Claude Code, " \
                       "then quickly resume them with 'cc <tag>' from anywhere. " \
                       "v1.4.1: Proactive bookmark migration on context continuations via hook."
  spec.homepage      = "https://github.com/isene/CC-sessions"
  spec.license       = "Unlicense"

  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/blob/main/CHANGELOG.md"

  spec.files = Dir[
    "bin/*",
    "commands/*",
    "lib/**/*",
    "LICENSE",
    "README.md",
    "CHANGELOG.md"
  ]

  spec.bindir        = "bin"
  spec.executables   = ["cc", "cc-bookmark"]

end
