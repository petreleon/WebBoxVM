#!/bin/sh
set -eu
test "$(dpkg --print-architecture)" = arm64
mkdir -p /output/packages /output/manifests
dpkg-query -W -f='${Package}\t${Version}\t${Architecture}\n' > /output/manifests/base-packages.tsv
rm -f /etc/apt/sources.list.d/debian.sources /etc/apt/apt.conf.d/docker-clean
cat > /etc/apt/sources.list <<'EOF'
deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/20260901T000000Z/ trixie main
deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/20260901T000000Z/ trixie-security main
EOF
cat > /etc/apt/preferences.d/i01-snapshot <<'EOF'
Package: *
Pin: origin "snapshot.debian.org"
Pin-Priority: 1001
EOF
apt-get update
apt-cache policy linux-image-6.12.94+deb13-arm64 > /output/manifests/kernel-policy.txt
: > /output/manifests/requested-packages.txt
for package in "$@"; do
    version=$(apt-cache policy "$package" | sed -n 's/^  Candidate: //p')
    test -n "$version" && test "$version" != '(none)'
    printf '%s=%s\n' "$package" "$version" >> /output/manifests/requested-packages.txt
done
set -- $(cat /output/manifests/requested-packages.txt)
apt-get --allow-downgrades --print-uris --download-only --no-install-recommends install "$@" > /output/manifests/package-uris.txt
apt-get -y --allow-downgrades --download-only --no-install-recommends -o Dir::Cache::archives=/output/packages install "$@"
: > /output/manifests/downloaded-packages.tsv
for package in /output/packages/*.deb; do
    printf '%s\t%s\t' "$(basename "$package")" "$(stat -c %s "$package")" >> /output/manifests/downloaded-packages.tsv
    dpkg-deb -W --showformat='${Package}\t${Version}\t${Architecture}\t' "$package" >> /output/manifests/downloaded-packages.tsv
    sha256sum "$package" | cut -d ' ' -f 1 >> /output/manifests/downloaded-packages.tsv
done
if apt-cache show linux-image-6.12.94+deb13-arm64=6.12.94-1 > /output/manifests/kernel-package.txt 2>/output/manifests/kernel-package-error.txt; then
    mkdir -p /output/kernel-package
    cd /output/kernel-package
    apt-get download linux-image-6.12.94+deb13-arm64=6.12.94-1
    sha256sum ./*.deb > /output/manifests/kernel-package.sha256
fi
mkdir -p /output/apt-lists
cp -r --no-preserve=ownership /var/lib/apt/lists/. /output/apt-lists/
