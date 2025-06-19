README.md
markdown# 🗺️ Simulation de Carte Procédurale en Rust

## Description
Ce projet est une simulation de carte 2D avec génération procédurale d'obstacles et de ressources, développée en Rust et accessible via navigateur web.

## Fonctionnalités
- ✅ Génération procédurale avec Perlin noise
- ✅ Seed pour reproduction des cartes
- ✅ 3 types de ressources : Énergie, Minerais, Points scientifiques
- ✅ Interface web pour visualisation
- ✅ Dockerisé (pas besoin d'installer Rust)
- ✅ Code clean et modulaire

## Prérequis
- Docker et Docker Compose installés
- Un navigateur web moderne

## Installation et lancement

1. **Créer la structure du projet** :
\`\`\`bash
mkdir rust-map-simulation
cd rust-map-simulation
\`\`\`

2. **Créer les dossiers nécessaires** :
\`\`\`bash
mkdir -p src/map src/static
\`\`\`

3. **Copier tous les fichiers** dans leurs emplacements respectifs :
- \`Dockerfile\` à la racine
- \`docker-compose.yml\` à la racine
- \`Cargo.toml\` à la racine
- \`src/main.rs\`
- \`src/map/mod.rs\`
- \`src/map/generator.rs\`
- \`src/map/resources.rs\`
- \`src/static/index.html\`

4. **Lancer l'application** :
\`\`\`bash
docker-compose up --build
\`\`\`

5. **Accéder à l'interface** :
Ouvrez votre navigateur à l'adresse : http://localhost:8080

## Utilisation

### Interface web
- **Seed** : Nombre pour générer toujours la même carte
- **Largeur/Hauteur** : Dimensions de la carte (10-100)
- **Générer la carte** : Crée une nouvelle carte avec les paramètres

### API REST
\`\`\`bash
# Générer une carte
GET http://localhost:8080/api/map?seed=42&width=50&height=50
\`\`\`

## Architecture du code

### Structure modulaire
\`\`\`
src/
├── main.rs          # Point d'entrée et serveur web
├── map/
│   ├── mod.rs       # Types et structures principales
│   ├── generator.rs # Génération procédurale
│   └── resources.rs # Gestion des ressources
└── static/
    └── index.html   # Interface web
\`\`\`

### Principes Clean Code appliqués
- **Single Responsibility** : Chaque module a une responsabilité unique
- **DRY** : Pas de duplication de code
- **KISS** : Solutions simples et compréhensibles
- **Nommage explicite** : Variables et fonctions auto-documentées

## Développement

### Modifications en temps réel
Le docker-compose utilise \`cargo watch\` pour recompiler automatiquement lors des modifications.

### Ajouter des fonctionnalités
1. Modifier le code Rust
2. Les changements sont automatiquement détectés et recompilés
3. Rafraîchir le navigateur pour voir les changements

## Ressources de la carte

- **Énergie** (jaune) : Consommable, quantité 10-50
- **Minerais** (bleu) : Consommable, quantité 5-25  
- **Points scientifiques** (vert) : Non consommable

## Personnalisation

### Modifier les paramètres de génération
Dans \`main.rs\`, ajuster le \`MapConfig\` :
\`\`\`rust
MapConfig {
    obstacle_threshold: 0.3,  // Densité des obstacles
    energy_density: 0.05,     // Densité de l'énergie
    mineral_density: 0.03,    // Densité des minerais
    science_density: 0.02,    // Densité des points scientifiques
}
\`\`\`

## Arrêt de l'application
\`\`\`bash
docker-compose down
\`\`\`

🚀 Instructions de mise en place

Créer le dossier racine :

bashmkdir rust-map-simulation && cd rust-map-simulation

Créer la structure :

bashmkdir -p src/map src/static

Créer tous les fichiers en copiant le contenu ci-dessus dans les fichiers correspondants
Lancer l'application :

bashdocker-compose up --build

Ouvrir le navigateur à http://localhost:8080

C'est tout ! Le projet est maintenant prêt à être utilisé et présenté à vos étudiants. 🎉