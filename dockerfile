FROM rust:1.82

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

# Compile 
RUN touch src/main.rs && \
    cargo build --release

EXPOSE 8080

# lancement de app!!!
CMD ["cargo", "run", "--release"]