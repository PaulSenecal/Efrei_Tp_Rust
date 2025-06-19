# Utilise l'image officielle Rust
FROM rust:1.75

# Installe les dépendances système
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Définit le répertoire de travail
WORKDIR /app

# Copie les fichiers de configuration Cargo
COPY Cargo.toml ./

# Pré-compile les dépendances
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copie le code source
COPY . .

# Compile l'application
RUN touch src/main.rs && \
    cargo build --release

# Expose le port 8080
EXPOSE 8080

# Lance l'application
CMD ["cargo", "run", "--release"]