#!/usr/bin/env bash
# Run parity/cpan_topn/smoke_all.pl under pe(1) with -I local/lib/perl5 (pe ignores PERL5LIB).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCAL_LIB="$ROOT/parity/cpan_topn/local/lib/perl5"
PE="${PE:-$ROOT/target/release/pe}"
SMOKE="$ROOT/parity/cpan_topn/smoke_all.pl"

export LC_ALL=C
export LANG=C

if [[ ! -d "$LOCAL_LIB" ]]; then
  echo "cpan_topn: missing $LOCAL_LIB — run: bash parity/cpan_topn/install_deps.sh" >&2
  exit 2
fi

if [[ ! -x "$PE" ]]; then
  echo "cpan_topn: building release pe …" >&2
  (builtin cd "$ROOT" && cargo build --release --locked -q)
fi

if [[ ! -x "$PE" ]]; then
  echo "cpan_topn: no executable at PE=$PE" >&2
  exit 2
fi

# pe(1) does not read PERL5LIB — prepend the local lib with -I (before default vendor/perl).
echo "cpan_topn: PE=$PE -I $LOCAL_LIB …" >&2

"$PE" -I "$LOCAL_LIB" "$SMOKE"
echo "cpan_topn: smoke_all.pl OK" >&2
