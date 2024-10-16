FROM golang:1.23

WORKDIR /

# Install archway
RUN git clone https://github.com/archway-network/archway.git \
    && cd ./archway  \
    && git checkout tags/v9.0.0-rc3 \
    && make install

# Install Hermes
RUN curl -L "https://github.com/informalsystems/hermes/releases/download/v1.10.3/hermes-v1.10.3-x86_64-unknown-linux-gnu.tar.gz" > "hermes.tar.gz" \
    && mkdir -p ./.hermes/bin/ && tar -C ./.hermes/bin/ -vxzf hermes.tar.gz

# Install GM deps
RUN apt-get update && apt-get install jq less -y
RUN curl -Lo /usr/local/bin/sconfig https://github.com/freshautomations/sconfig/releases/download/v0.1.0/sconfig_linux_amd64 \
    && curl -Lo /usr/local/bin/stoml https://github.com/freshautomations/stoml/releases/download/v0.7.0/stoml_linux_amd64 \
    && chmod 755 /usr/local/bin/sconfig && chmod 755 /usr/local/bin/stoml

# Install GM
ADD "https://www.random.org/cgi-bin/randbyte?nbytes=10&format=h" skipcache
RUN git clone https://github.com/FloppyDisck/gm && gm/bin/gm install

# Expose chain node ports
EXPOSE 27010 27011 27012
EXPOSE 27020 27021 27022

COPY scripts/init_network.sh .
RUN chmod +x ./init_network.sh

# Setup GM config
COPY scripts/gm.toml /root/.gm/gm.toml

# Setup hermes config and keys
ENV USER_MNEMONIC="any giant turtle pioneer frequent frown harvest ancient episode junior vocal rent shrimp icon idle echo suspect clean cage eternal sample post heavy enough"
COPY scripts/relayer_config.toml ./.hermes/config.toml
RUN echo "${USER_MNEMONIC}" > ./mnemonic.txt  \
    && ./.hermes/bin/hermes --config ./.hermes/config.toml keys add --chain archway-1 --mnemonic-file ./mnemonic.txt \
    && ./.hermes/bin/hermes --config ./.hermes/config.toml keys add --chain archway-2 --mnemonic-file ./mnemonic.txt

ENTRYPOINT ./init_network.sh