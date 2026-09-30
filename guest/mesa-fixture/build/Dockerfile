FROM debian@sha256:da496358bd6934d2bd6a563a33176a2e50eff5490c54b4ac6fb051b69fef4071
COPY packages/ /input/packages/
COPY packages.sha256 /input/packages.sha256
RUN cd /input/packages && sha256sum -c /input/packages.sha256 && \
    apt-get -y --allow-downgrades --no-install-recommends install /input/packages/*.deb && \
    dpkg-query -W -f='${Package}\t${Version}\t${Architecture}\n' > /builder-packages.tsv && \
    rm -r /input
ENV PKG_CONFIG_PATH=/opt/mesa-f02/lib/pkgconfig
ENV LD_LIBRARY_PATH=/opt/mesa-f02/lib
WORKDIR /work
