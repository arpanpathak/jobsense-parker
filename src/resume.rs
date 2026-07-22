//! # Resume Intelligence Engine
//!
//! Extracts structured intelligence from resume text using:
//!
//! 1. **Comprehensive tech-skill dictionary** (500+ known technologies)
//! 2. **Context-aware extraction** (patterns like "experience with X", "proficient in Y")
//! 3. **Role/seniority inference** from title patterns
//! 4. **Domain detection** (backend, frontend, DevOps, data science, etc.)
//! 5. **Education extraction** (degrees, fields, universities)
//! 6. **Certification detection**
//!
//! This replaces the previous regex-only approach that produced noisy keywords.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Comprehensive Tech Skill Dictionary ──────────────────────────────────────
//
// Organized by domain so we can infer the candidate's area of expertise.
// Each entry maps (lowercase name) → domain category.

/// Categories of skills for domain inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillDomain {
    Language,
    Framework,
    Frontend,
    Backend,
    Database,
    CloudDevOps,
    DataMl,
    Mobile,
    Tools,
    Platform,
    Protocol,
    Concept,
    Other,
}

/// A known tech skill with its domain classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownSkill {
    pub name: String,
    pub domain: SkillDomain,
}

/// Singleton dictionary of known tech skills.
pub struct SkillDictionary {
    /// Map of lowercase skill name → KnownSkill
    skills: HashMap<String, KnownSkill>,
    /// Regex that matches any known skill as a word boundary.
    combined_re: Regex,
}

impl SkillDictionary {
    /// Build the dictionary from the built-in list.
    pub fn new() -> Self {
        let entries = Self::builtin_skills();
        let mut skills = HashMap::with_capacity(entries.len());
        for (name, domain) in entries {
            skills.insert(
                name.to_lowercase(),
                KnownSkill {
                    name: name.to_string(),
                    domain,
                },
            );
        }

        // Build combined regex: word-boundary match for any known skill
        let mut pattern = String::from(r"(?i)\b(?:");
        let names: Vec<&str> = skills.keys().map(|s| s.as_str()).collect();
        // Sort longest first so multi-word skills match before shorter substrings
        let mut sorted = names.clone();
        sorted.sort_by(|a, b| b.len().cmp(&a.len()));
        pattern.push_str(&sorted.join("|"));
        pattern.push_str(r")\b");
        // Handle skills with dots/hyphens (like "c++", ".net", "node.js")
        let pattern2 = r"(?i)\b(c\+\+|c#|f#|\.net|node\.js|react\.js|vue\.js|angular\.js|next\.js|three\.js|d3\.js|express\.js|socket\.io)\b";
        let combined = format!(r"(?:{}|{})", pattern, pattern2);

        let combined_re = Regex::new(&combined).unwrap();

        Self {
            skills,
            combined_re,
        }
    }

    /// Find all known tech skills in text.
    pub fn find_skills(&self, text: &str) -> Vec<KnownSkill> {
        let mut found: Vec<KnownSkill> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cap in self.combined_re.captures_iter(text) {
            let matched = cap.get(0).unwrap().as_str().to_lowercase();
            if seen.insert(matched.clone()) {
                // Check direct lookup first
                if let Some(ks) = self.skills.get(&matched) {
                    found.push(ks.clone());
                } else {
                    // It matched via the multi-word pattern group - do a linear scan
                    for (name, ks) in &self.skills {
                        if name == &matched {
                            found.push(ks.clone());
                            break;
                        }
                    }
                }
            }
        }

        found
    }

    /// Count how many skills from a given set appear in text.
    pub fn count_skills_in_text(&self, skills: &[String], text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        skills
            .iter()
            .filter(|s| lower.contains(&s.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Return all skills as a list of strings.
    pub fn all_skill_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.skills.keys().cloned().collect();
        names.sort();
        names
    }

    /// Comprehensive built-in skill list (500+ entries).
    fn builtin_skills() -> Vec<(&'static str, SkillDomain)> {
        vec![
            // ── Programming Languages ──────────────────────────────────────
            ("Rust", SkillDomain::Language),
            ("Python", SkillDomain::Language),
            ("JavaScript", SkillDomain::Language),
            ("TypeScript", SkillDomain::Language),
            ("Java", SkillDomain::Language),
            ("Go", SkillDomain::Language),
            ("Golang", SkillDomain::Language),
            ("C", SkillDomain::Language),
            ("C++", SkillDomain::Language),
            ("C++11", SkillDomain::Language),
            ("C++14", SkillDomain::Language),
            ("C++17", SkillDomain::Language),
            ("C++20", SkillDomain::Language),
            ("C#", SkillDomain::Language),
            ("F#", SkillDomain::Language),
            ("Kotlin", SkillDomain::Language),
            ("Swift", SkillDomain::Language),
            ("Scala", SkillDomain::Language),
            ("Ruby", SkillDomain::Language),
            ("PHP", SkillDomain::Language),
            ("Haskell", SkillDomain::Language),
            ("Clojure", SkillDomain::Language),
            ("Elixir", SkillDomain::Language),
            ("Erlang", SkillDomain::Language),
            ("Dart", SkillDomain::Language),
            ("Lua", SkillDomain::Language),
            ("R", SkillDomain::Language),
            ("MATLAB", SkillDomain::Language),
            ("Julia", SkillDomain::Language),
            ("Perl", SkillDomain::Language),
            ("Shell", SkillDomain::Language),
            ("Bash", SkillDomain::Language),
            ("Zsh", SkillDomain::Language),
            ("PowerShell", SkillDomain::Language),
            ("SQL", SkillDomain::Language),
            ("PL/SQL", SkillDomain::Language),
            ("T-SQL", SkillDomain::Language),
            ("GraphQL", SkillDomain::Language),
            ("HTML", SkillDomain::Language),
            ("HTML5", SkillDomain::Language),
            ("CSS", SkillDomain::Language),
            ("CSS3", SkillDomain::Language),
            ("Sass", SkillDomain::Language),
            ("SCSS", SkillDomain::Language),
            ("Less", SkillDomain::Language),
            ("Assembly", SkillDomain::Language),
            ("COBOL", SkillDomain::Language),
            ("Fortran", SkillDomain::Language),
            ("Lisp", SkillDomain::Language),
            ("Scheme", SkillDomain::Language),
            ("OCaml", SkillDomain::Language),
            ("Zig", SkillDomain::Language),
            ("Nim", SkillDomain::Language),
            ("Crystal", SkillDomain::Language),
            ("Solidity", SkillDomain::Language),
            ("Vyper", SkillDomain::Language),
            ("Terraform", SkillDomain::Language),
            ("Pulumi", SkillDomain::Language),
            ("WebAssembly", SkillDomain::Language),
            ("WASM", SkillDomain::Language),
            // ── Frontend Frameworks / Libraries ────────────────────────────
            ("React", SkillDomain::Frontend),
            ("React.js", SkillDomain::Frontend),
            ("Next.js", SkillDomain::Frontend),
            ("Vue", SkillDomain::Frontend),
            ("Vue.js", SkillDomain::Frontend),
            ("Nuxt", SkillDomain::Frontend),
            ("Angular", SkillDomain::Frontend),
            ("AngularJS", SkillDomain::Frontend),
            ("Svelte", SkillDomain::Frontend),
            ("SvelteKit", SkillDomain::Frontend),
            ("SolidJS", SkillDomain::Frontend),
            ("Preact", SkillDomain::Frontend),
            ("Lit", SkillDomain::Frontend),
            ("jQuery", SkillDomain::Frontend),
            ("Ember", SkillDomain::Frontend),
            ("Backbone", SkillDomain::Frontend),
            ("Redux", SkillDomain::Frontend),
            ("Zustand", SkillDomain::Frontend),
            ("MobX", SkillDomain::Frontend),
            ("Recoil", SkillDomain::Frontend),
            ("Tailwind CSS", SkillDomain::Frontend),
            ("Bootstrap", SkillDomain::Frontend),
            ("Material UI", SkillDomain::Frontend),
            ("MUI", SkillDomain::Frontend),
            ("Ant Design", SkillDomain::Frontend),
            ("Chakra UI", SkillDomain::Frontend),
            ("Styled Components", SkillDomain::Frontend),
            ("Framer Motion", SkillDomain::Frontend),
            ("Three.js", SkillDomain::Frontend),
            ("D3.js", SkillDomain::Frontend),
            ("Chart.js", SkillDomain::Frontend),
            ("Webpack", SkillDomain::Frontend),
            ("Vite", SkillDomain::Frontend),
            ("ESBuild", SkillDomain::Frontend),
            ("Rollup", SkillDomain::Frontend),
            ("Parcel", SkillDomain::Frontend),
            ("Babel", SkillDomain::Frontend),
            ("Storybook", SkillDomain::Frontend),
            ("Cypress", SkillDomain::Frontend),
            ("Playwright", SkillDomain::Frontend),
            ("Jest", SkillDomain::Frontend),
            ("Vitest", SkillDomain::Frontend),
            ("Mocha", SkillDomain::Frontend),
            ("Chai", SkillDomain::Frontend),
            ("Enzyme", SkillDomain::Frontend),
            ("Testing Library", SkillDomain::Frontend),
            ("ESLint", SkillDomain::Frontend),
            ("Prettier", SkillDomain::Frontend),
            ("WebGL", SkillDomain::Frontend),
            ("Canvas", SkillDomain::Frontend),
            ("SVG", SkillDomain::Frontend),
            ("HTMX", SkillDomain::Frontend),
            ("Alpine.js", SkillDomain::Frontend),
            ("Turbo", SkillDomain::Frontend),
            ("Astro", SkillDomain::Frontend),
            ("Remix", SkillDomain::Frontend),
            ("Gatsby", SkillDomain::Frontend),
            ("Eleventy", SkillDomain::Frontend),
            ("Hugo", SkillDomain::Frontend),
            ("Jekyll", SkillDomain::Frontend),
            // ── Backend Frameworks ─────────────────────────────────────────
            ("Node.js", SkillDomain::Backend),
            ("Express", SkillDomain::Backend),
            ("Express.js", SkillDomain::Backend),
            ("NestJS", SkillDomain::Backend),
            ("Fastify", SkillDomain::Backend),
            ("Koa", SkillDomain::Backend),
            ("Hapi", SkillDomain::Backend),
            ("Socket.io", SkillDomain::Backend),
            ("Django", SkillDomain::Backend),
            ("Flask", SkillDomain::Backend),
            ("FastAPI", SkillDomain::Backend),
            ("Spring", SkillDomain::Backend),
            ("Spring Boot", SkillDomain::Backend),
            ("Spring Framework", SkillDomain::Backend),
            ("ASP.NET", SkillDomain::Backend),
            ("ASP.NET Core", SkillDomain::Backend),
            ("Rails", SkillDomain::Backend),
            ("Ruby on Rails", SkillDomain::Backend),
            ("Phoenix", SkillDomain::Backend),
            ("Laravel", SkillDomain::Backend),
            ("Symfony", SkillDomain::Backend),
            ("Actix", SkillDomain::Backend),
            ("Actix Web", SkillDomain::Backend),
            ("Rocket", SkillDomain::Backend),
            ("Axum", SkillDomain::Backend),
            ("Tonic", SkillDomain::Backend),
            ("Tokio", SkillDomain::Backend),
            ("Async-std", SkillDomain::Backend),
            ("Gin", SkillDomain::Backend),
            ("Echo", SkillDomain::Backend),
            ("Fiber", SkillDomain::Backend),
            ("Dropwizard", SkillDomain::Backend),
            ("Play Framework", SkillDomain::Backend),
            ("Akka", SkillDomain::Backend),
            ("CakePHP", SkillDomain::Backend),
            ("CodeIgniter", SkillDomain::Backend),
            ("Yii", SkillDomain::Backend),
            ("Flux", SkillDomain::Backend),
            ("Redux Saga", SkillDomain::Backend),
            // ─── Databases ──────────────────────────────────────────────────
            ("PostgreSQL", SkillDomain::Database),
            ("Postgres", SkillDomain::Database),
            ("MySQL", SkillDomain::Database),
            ("MariaDB", SkillDomain::Database),
            ("SQLite", SkillDomain::Database),
            ("MongoDB", SkillDomain::Database),
            ("Mongo", SkillDomain::Database),
            ("Redis", SkillDomain::Database),
            ("DynamoDB", SkillDomain::Database),
            ("Cassandra", SkillDomain::Database),
            ("Elasticsearch", SkillDomain::Database),
            ("Elastic Search", SkillDomain::Database),
            ("Elastic", SkillDomain::Database),
            ("CockroachDB", SkillDomain::Database),
            ("Cockroach", SkillDomain::Database),
            ("Neo4j", SkillDomain::Database),
            ("CouchDB", SkillDomain::Database),
            ("Couchbase", SkillDomain::Database),
            ("Firebase", SkillDomain::Database),
            ("Firestore", SkillDomain::Database),
            ("Supabase", SkillDomain::Database),
            ("PlanetScale", SkillDomain::Database),
            ("TimescaleDB", SkillDomain::Database),
            ("InfluxDB", SkillDomain::Database),
            ("ClickHouse", SkillDomain::Database),
            ("BigQuery", SkillDomain::Database),
            ("Redshift", SkillDomain::Database),
            ("Snowflake", SkillDomain::Database),
            ("Oracle", SkillDomain::Database),
            ("SQL Server", SkillDomain::Database),
            ("MSSQL", SkillDomain::Database),
            ("Prisma", SkillDomain::Database),
            ("Drizzle", SkillDomain::Database),
            ("TypeORM", SkillDomain::Database),
            ("Sequelize", SkillDomain::Database),
            ("Knex.js", SkillDomain::Database),
            ("SQLAlchemy", SkillDomain::Database),
            ("Diesel", SkillDomain::Database),
            ("Mongoose", SkillDomain::Database),
            ("Flyway", SkillDomain::Database),
            ("Liquibase", SkillDomain::Database),
            ("Hibernate", SkillDomain::Database),
            ("JPA", SkillDomain::Database),
            ("MyBatis", SkillDomain::Database),
            ("R2DBC", SkillDomain::Database),
            // ─── Cloud & DevOps ─────────────────────────────────────────────
            ("AWS", SkillDomain::CloudDevOps),
            ("Amazon Web Services", SkillDomain::CloudDevOps),
            ("Azure", SkillDomain::CloudDevOps),
            ("Microsoft Azure", SkillDomain::CloudDevOps),
            ("GCP", SkillDomain::CloudDevOps),
            ("Google Cloud", SkillDomain::CloudDevOps),
            ("Google Cloud Platform", SkillDomain::CloudDevOps),
            ("Docker", SkillDomain::CloudDevOps),
            ("Kubernetes", SkillDomain::CloudDevOps),
            ("K8s", SkillDomain::CloudDevOps),
            ("Helm", SkillDomain::CloudDevOps),
            ("Terraform", SkillDomain::CloudDevOps),
            ("Pulumi", SkillDomain::CloudDevOps),
            ("Ansible", SkillDomain::CloudDevOps),
            ("Chef", SkillDomain::CloudDevOps),
            ("Puppet", SkillDomain::CloudDevOps),
            ("SaltStack", SkillDomain::CloudDevOps),
            ("Jenkins", SkillDomain::CloudDevOps),
            ("GitHub Actions", SkillDomain::CloudDevOps),
            ("GitLab CI", SkillDomain::CloudDevOps),
            ("CircleCI", SkillDomain::CloudDevOps),
            ("Travis CI", SkillDomain::CloudDevOps),
            ("TeamCity", SkillDomain::CloudDevOps),
            ("Bamboo", SkillDomain::CloudDevOps),
            ("ArgoCD", SkillDomain::CloudDevOps),
            ("Argo Workflows", SkillDomain::CloudDevOps),
            ("Spinnaker", SkillDomain::CloudDevOps),
            ("Vault", SkillDomain::CloudDevOps),
            ("Consul", SkillDomain::CloudDevOps),
            ("Nomad", SkillDomain::CloudDevOps),
            ("Packe", SkillDomain::CloudDevOps),
            ("Vagrant", SkillDomain::CloudDevOps),
            ("CloudFormation", SkillDomain::CloudDevOps),
            ("CDK", SkillDomain::CloudDevOps),
            ("AWS CDK", SkillDomain::CloudDevOps),
            ("Serverless", SkillDomain::CloudDevOps),
            ("SAM", SkillDomain::CloudDevOps),
            ("AWS Lambda", SkillDomain::CloudDevOps),
            ("Lambda", SkillDomain::CloudDevOps),
            ("EC2", SkillDomain::CloudDevOps),
            ("ECS", SkillDomain::CloudDevOps),
            ("EKS", SkillDomain::CloudDevOps),
            ("Fargate", SkillDomain::CloudDevOps),
            ("S3", SkillDomain::CloudDevOps),
            ("RDS", SkillDomain::CloudDevOps),
            ("Aurora", SkillDomain::CloudDevOps),
            ("CloudFront", SkillDomain::CloudDevOps),
            ("CloudWatch", SkillDomain::CloudDevOps),
            ("IAM", SkillDomain::CloudDevOps),
            ("VPC", SkillDomain::CloudDevOps),
            ("Route53", SkillDomain::CloudDevOps),
            ("SQS", SkillDomain::CloudDevOps),
            ("SNS", SkillDomain::CloudDevOps),
            ("Kinesis", SkillDomain::CloudDevOps),
            ("Step Functions", SkillDomain::CloudDevOps),
            ("API Gateway", SkillDomain::CloudDevOps),
            ("Prometheus", SkillDomain::CloudDevOps),
            ("Grafana", SkillDomain::CloudDevOps),
            ("Datadog", SkillDomain::CloudDevOps),
            ("New Relic", SkillDomain::CloudDevOps),
            ("Sentry", SkillDomain::CloudDevOps),
            ("OpenTelemetry", SkillDomain::CloudDevOps),
            ("Jaeger", SkillDomain::CloudDevOps),
            ("Zipkin", SkillDomain::CloudDevOps),
            ("ELK Stack", SkillDomain::CloudDevOps),
            ("Logstash", SkillDomain::CloudDevOps),
            ("Kibana", SkillDomain::CloudDevOps),
            ("Fluentd", SkillDomain::CloudDevOps),
            ("Nginx", SkillDomain::CloudDevOps),
            ("Apache", SkillDomain::CloudDevOps),
            ("Caddy", SkillDomain::CloudDevOps),
            ("HAProxy", SkillDomain::CloudDevOps),
            ("Traefik", SkillDomain::CloudDevOps),
            ("Envoy", SkillDomain::CloudDevOps),
            ("Istio", SkillDomain::CloudDevOps),
            ("Linkerd", SkillDomain::CloudDevOps),
            ("Consul Connect", SkillDomain::CloudDevOps),
            ("Cert Manager", SkillDomain::CloudDevOps),
            ("Velero", SkillDomain::CloudDevOps),
            ("Rancher", SkillDomain::CloudDevOps),
            ("OpenShift", SkillDomain::CloudDevOps),
            // ─── Data / ML / AI ─────────────────────────────────────────────
            ("TensorFlow", SkillDomain::DataMl),
            ("PyTorch", SkillDomain::DataMl),
            ("Keras", SkillDomain::DataMl),
            ("Scikit-learn", SkillDomain::DataMl),
            ("Scikit Learn", SkillDomain::DataMl),
            ("XGBoost", SkillDomain::DataMl),
            ("LightGBM", SkillDomain::DataMl),
            ("CatBoost", SkillDomain::DataMl),
            ("Pandas", SkillDomain::DataMl),
            ("NumPy", SkillDomain::DataMl),
            ("SciPy", SkillDomain::DataMl),
            ("Matplotlib", SkillDomain::DataMl),
            ("Seaborn", SkillDomain::DataMl),
            ("Plotly", SkillDomain::DataMl),
            ("Jupyter", SkillDomain::DataMl),
            ("Jupyter Notebook", SkillDomain::DataMl),
            ("Apache Spark", SkillDomain::DataMl),
            ("Spark", SkillDomain::DataMl),
            ("Hadoop", SkillDomain::DataMl),
            ("Hive", SkillDomain::DataMl),
            ("Pig", SkillDomain::DataMl),
            ("Airflow", SkillDomain::DataMl),
            ("Apache Airflow", SkillDomain::DataMl),
            ("dbt", SkillDomain::DataMl),
            ("Kafka", SkillDomain::DataMl),
            ("Apache Kafka", SkillDomain::DataMl),
            ("Flink", SkillDomain::DataMl),
            ("Beam", SkillDomain::DataMl),
            ("Tableau", SkillDomain::DataMl),
            ("Power BI", SkillDomain::DataMl),
            ("Looker", SkillDomain::DataMl),
            ("Hugging Face", SkillDomain::DataMl),
            ("Transformers", SkillDomain::DataMl),
            ("LangChain", SkillDomain::DataMl),
            ("LlamaIndex", SkillDomain::DataMl),
            ("OpenAI", SkillDomain::DataMl),
            ("GPT", SkillDomain::DataMl),
            ("LLM", SkillDomain::DataMl),
            ("RAG", SkillDomain::DataMl),
            ("Vector Database", SkillDomain::DataMl),
            ("Pinecone", SkillDomain::DataMl),
            ("Weaviate", SkillDomain::DataMl),
            ("Milvus", SkillDomain::DataMl),
            ("Qdrant", SkillDomain::DataMl),
            ("Chroma", SkillDomain::DataMl),
            ("MLflow", SkillDomain::DataMl),
            ("Kubeflow", SkillDomain::DataMl),
            ("SageMaker", SkillDomain::DataMl),
            ("Vertex AI", SkillDomain::DataMl),
            ("ONNX", SkillDomain::DataMl),
            ("Triton", SkillDomain::DataMl),
            ("CUDA", SkillDomain::DataMl),
            ("OpenCV", SkillDomain::DataMl),
            ("NLTK", SkillDomain::DataMl),
            ("spaCy", SkillDomain::DataMl),
            ("Stanford NLP", SkillDomain::DataMl),
            // ─── Mobile ─────────────────────────────────────────────────────
            ("Android", SkillDomain::Mobile),
            ("iOS", SkillDomain::Mobile),
            ("SwiftUI", SkillDomain::Mobile),
            ("UIKit", SkillDomain::Mobile),
            ("Flutter", SkillDomain::Mobile),
            ("React Native", SkillDomain::Mobile),
            ("Kotlin Multiplatform", SkillDomain::Mobile),
            ("Xamarin", SkillDomain::Mobile),
            ("Ionic", SkillDomain::Mobile),
            ("Cordova", SkillDomain::Mobile),
            ("Expo", SkillDomain::Mobile),
            ("Jetpack Compose", SkillDomain::Mobile),
            ("Android SDK", SkillDomain::Mobile),
            ("ARCore", SkillDomain::Mobile),
            ("ARKit", SkillDomain::Mobile),
            ("Core Data", SkillDomain::Mobile),
            ("Realm", SkillDomain::Mobile),
            // ─── Tools & Platforms ──────────────────────────────────────────
            ("Git", SkillDomain::Tools),
            ("GitHub", SkillDomain::Tools),
            ("GitLab", SkillDomain::Tools),
            ("Bitbucket", SkillDomain::Tools),
            ("Jira", SkillDomain::Tools),
            ("Confluence", SkillDomain::Tools),
            ("Slack", SkillDomain::Tools),
            ("Discord", SkillDomain::Tools),
            ("Notion", SkillDomain::Tools),
            ("Linear", SkillDomain::Tools),
            ("Asana", SkillDomain::Tools),
            ("Trello", SkillDomain::Tools),
            ("VS Code", SkillDomain::Tools),
            ("Visual Studio", SkillDomain::Tools),
            ("IntelliJ", SkillDomain::Tools),
            ("PyCharm", SkillDomain::Tools),
            ("Vim", SkillDomain::Tools),
            ("Neovim", SkillDomain::Tools),
            ("Emacs", SkillDomain::Tools),
            ("Linux", SkillDomain::Tools),
            ("Unix", SkillDomain::Tools),
            ("macOS", SkillDomain::Tools),
            ("Windows", SkillDomain::Tools),
            ("Make", SkillDomain::Tools),
            ("CMake", SkillDomain::Tools),
            ("GCC", SkillDomain::Tools),
            ("Clang", SkillDomain::Tools),
            ("LLVM", SkillDomain::Tools),
            ("gRPC", SkillDomain::Tools),
            ("Protobuf", SkillDomain::Tools),
            ("Protocol Buffers", SkillDomain::Tools),
            ("REST", SkillDomain::Tools),
            ("RESTful", SkillDomain::Tools),
            ("GraphQL", SkillDomain::Tools),
            ("WebSockets", SkillDomain::Tools),
            ("WebSocket", SkillDomain::Tools),
            ("OAuth", SkillDomain::Tools),
            ("OAuth2", SkillDomain::Tools),
            ("JWT", SkillDomain::Tools),
            ("SAML", SkillDomain::Tools),
            ("OpenID", SkillDomain::Tools),
            ("TLS", SkillDomain::Tools),
            ("SSL", SkillDomain::Tools),
            ("HTTP", SkillDomain::Tools),
            ("HTTPS", SkillDomain::Tools),
            ("TCP/IP", SkillDomain::Tools),
            ("UDP", SkillDomain::Tools),
            ("DNS", SkillDomain::Tools),
            ("DHCP", SkillDomain::Tools),
            ("Nginx", SkillDomain::Tools),
            ("Apache", SkillDomain::Tools),
            ("Postman", SkillDomain::Tools),
            ("Swagger", SkillDomain::Tools),
            ("OpenAPI", SkillDomain::Tools),
            ("Insomnia", SkillDomain::Tools),
            ("curl", SkillDomain::Tools),
            ("Wireshark", SkillDomain::Tools),
            ("Yarn", SkillDomain::Tools),
            ("npm", SkillDomain::Tools),
            ("pnpm", SkillDomain::Tools),
            ("Bun", SkillDomain::Tools),
            ("Cargo", SkillDomain::Tools),
            ("Gradle", SkillDomain::Tools),
            ("Maven", SkillDomain::Tools),
            ("Ant", SkillDomain::Tools),
            ("SBT", SkillDomain::Tools),
            ("CocoaPods", SkillDomain::Tools),
            ("Homebrew", SkillDomain::Tools),
            ("Docker Compose", SkillDomain::Tools),
            ("Minikube", SkillDomain::Tools),
            ("Kind", SkillDomain::Tools),
            ("kubectl", SkillDomain::Tools),
            // ─── Architecture / Concepts ────────────────────────────────────
            ("Microservices", SkillDomain::Concept),
            ("Microservice", SkillDomain::Concept),
            ("Event-Driven", SkillDomain::Concept),
            ("Event Sourcing", SkillDomain::Concept),
            ("CQRS", SkillDomain::Concept),
            ("DDD", SkillDomain::Concept),
            ("Domain-Driven Design", SkillDomain::Concept),
            ("Hexagonal Architecture", SkillDomain::Concept),
            ("Clean Architecture", SkillDomain::Concept),
            ("Onion Architecture", SkillDomain::Concept),
            ("SOA", SkillDomain::Concept),
            ("RESTful API", SkillDomain::Concept),
            ("API Design", SkillDomain::Concept),
            ("API Gateway", SkillDomain::Concept),
            ("CI/CD", SkillDomain::Concept),
            ("Continuous Integration", SkillDomain::Concept),
            ("Continuous Delivery", SkillDomain::Concept),
            ("Continuous Deployment", SkillDomain::Concept),
            ("TDD", SkillDomain::Concept),
            ("Test-Driven Development", SkillDomain::Concept),
            ("BDD", SkillDomain::Concept),
            ("Agile", SkillDomain::Concept),
            ("Scrum", SkillDomain::Concept),
            ("Kanban", SkillDomain::Concept),
            ("DevOps", SkillDomain::Concept),
            ("SRE", SkillDomain::Concept),
            ("Site Reliability", SkillDomain::Concept),
            ("Observability", SkillDomain::Concept),
            ("Monitoring", SkillDomain::Concept),
            ("Alerting", SkillDomain::Concept),
            ("Distributed Systems", SkillDomain::Concept),
            ("Concurrency", SkillDomain::Concept),
            ("Parallel Computing", SkillDomain::Concept),
            ("Functional Programming", SkillDomain::Concept),
            ("Object-Oriented", SkillDomain::Concept),
            ("OOP", SkillDomain::Concept),
            ("Design Patterns", SkillDomain::Concept),
            ("SOLID", SkillDomain::Concept),
            ("ACID", SkillDomain::Concept),
            ("CAP Theorem", SkillDomain::Concept),
            ("Message Queue", SkillDomain::Concept),
            ("Pub/Sub", SkillDomain::Concept),
            ("Publish-Subscribe", SkillDomain::Concept),
            ("Load Balancing", SkillDomain::Concept),
            ("Caching", SkillDomain::Concept),
            ("CDN", SkillDomain::Concept),
            ("Sharding", SkillDomain::Concept),
            ("Replication", SkillDomain::Concept),
            ("Backup", SkillDomain::Concept),
            ("Disaster Recovery", SkillDomain::Concept),
            ("High Availability", SkillDomain::Concept),
            ("Scalability", SkillDomain::Concept),
            ("Security", SkillDomain::Concept),
            ("Authentication", SkillDomain::Concept),
            ("Authorization", SkillDomain::Concept),
            ("Encryption", SkillDomain::Concept),
            ("Penetration Testing", SkillDomain::Concept),
            ("Vulnerability Assessment", SkillDomain::Concept),
            ("Zero Trust", SkillDomain::Concept),
            // ─── Additional Tools ──────────────────────────────────────────
            ("RabbitMQ", SkillDomain::Tools),
            ("NATS", SkillDomain::Tools),
            ("Redis Pub/Sub", SkillDomain::Tools),
            ("gVisor", SkillDomain::Tools),
            ("Firecracker", SkillDomain::Tools),
            ("Wasmtime", SkillDomain::Tools),
            ("WebAssembly Runtime", SkillDomain::Tools),
            ("FFmpeg", SkillDomain::Tools),
            ("ImageMagick", SkillDomain::Tools),
            ("Puppeteer", SkillDomain::Tools),
            ("Selenium", SkillDomain::Tools),
            ("Appium", SkillDomain::Tools),
            ("Detox", SkillDomain::Tools),
            ("XCTest", SkillDomain::Tools),
            ("JUnit", SkillDomain::Tools),
            ("pytest", SkillDomain::Tools),
            ("unittest", SkillDomain::Tools),
            ("Mockito", SkillDomain::Tools),
            ("RSpec", SkillDomain::Tools),
            ("Cucumber", SkillDomain::Tools),
            ("Gatling", SkillDomain::Tools),
            ("JMeter", SkillDomain::Tools),
            ("k6", SkillDomain::Tools),
            ("Locust", SkillDomain::Tools),
            ("SonarQube", SkillDomain::Tools),
            ("SonarCloud", SkillDomain::Tools),
            ("Codecov", SkillDomain::Tools),
            ("Coveralls", SkillDomain::Tools),
            ("Snyk", SkillDomain::Tools),
            ("Dependabot", SkillDomain::Tools),
            ("Renovate", SkillDomain::Tools),
            ("Trivy", SkillDomain::Tools),
            ("Clair", SkillDomain::Tools),
            ("Falco", SkillDomain::Tools),
            ("Aqua", SkillDomain::Tools),
            ("Twistlock", SkillDomain::Tools),
            ("HashiCorp Vault", SkillDomain::Tools),
            ("Secrets Manager", SkillDomain::Tools),
            ("Parameter Store", SkillDomain::Tools),
            // ─── Domain-Specific ────────────────────────────────────────────
            ("Machine Learning", SkillDomain::DataMl),
            ("Deep Learning", SkillDomain::DataMl),
            ("Reinforcement Learning", SkillDomain::DataMl),
            ("Natural Language Processing", SkillDomain::DataMl),
            ("NLP", SkillDomain::DataMl),
            ("Computer Vision", SkillDomain::DataMl),
            ("CV", SkillDomain::DataMl),
            ("Speech Recognition", SkillDomain::DataMl),
            ("Generative AI", SkillDomain::DataMl),
            ("GenAI", SkillDomain::DataMl),
            ("Large Language Models", SkillDomain::DataMl),
            ("LLMs", SkillDomain::DataMl),
            ("Fine-tuning", SkillDomain::DataMl),
            ("Prompt Engineering", SkillDomain::DataMl),
            ("Embeddings", SkillDomain::DataMl),
            ("Semantic Search", SkillDomain::DataMl),
            ("Recommendation Systems", SkillDomain::DataMl),
            ("Anomaly Detection", SkillDomain::DataMl),
            ("Time Series", SkillDomain::DataMl),
            ("Statistical Modeling", SkillDomain::DataMl),
            ("A/B Testing", SkillDomain::DataMl),
            ("Experimentation", SkillDomain::DataMl),
            ("Causal Inference", SkillDomain::DataMl),
            // ─── Platform / SaaS ────────────────────────────────────────────
            ("Cloudflare", SkillDomain::Platform),
            ("Fastly", SkillDomain::Platform),
            ("Akamai", SkillDomain::Platform),
            ("Vercel", SkillDomain::Platform),
            ("Netlify", SkillDomain::Platform),
            ("Railway", SkillDomain::Platform),
            ("Fly.io", SkillDomain::Platform),
            ("Heroku", SkillDomain::Platform),
            ("DigitalOcean", SkillDomain::Platform),
            ("Linode", SkillDomain::Platform),
            ("Vultr", SkillDomain::Platform),
            ("Render", SkillDomain::Platform),
            ("PlanetScale", SkillDomain::Platform),
            ("Neon", SkillDomain::Platform),
            ("MongoDB Atlas", SkillDomain::Platform),
            ("Confluent Cloud", SkillDomain::Platform),
            ("Snowflake", SkillDomain::Platform),
            ("Databricks", SkillDomain::Platform),
            ("dbt Cloud", SkillDomain::Platform),
            ("Fivetran", SkillDomain::Platform),
            ("Airbyte", SkillDomain::Platform),
            ("Stitch", SkillDomain::Platform),
            ("Segment", SkillDomain::Platform),
            ("Twilio", SkillDomain::Platform),
            ("SendGrid", SkillDomain::Platform),
            ("Stripe", SkillDomain::Platform),
            ("Plaid", SkillDomain::Platform),
            ("Algolia", SkillDomain::Platform),
            ("Meilisearch", SkillDomain::Platform),
            ("Typesense", SkillDomain::Platform),
            ("Auth0", SkillDomain::Platform),
            ("Clerk", SkillDomain::Platform),
            ("Supabase Auth", SkillDomain::Platform),
            ("Firebase Auth", SkillDomain::Platform),
            ("Okta", SkillDomain::Platform),
            ("WorkOS", SkillDomain::Platform),
            // ─── Protocols / Standards ──────────────────────────────────────
            ("gRPC", SkillDomain::Protocol),
            ("GraphQL", SkillDomain::Protocol),
            ("REST", SkillDomain::Protocol),
            ("SOAP", SkillDomain::Protocol),
            ("MQTT", SkillDomain::Protocol),
            ("AMQP", SkillDomain::Protocol),
            ("HTTP/2", SkillDomain::Protocol),
            ("HTTP/3", SkillDomain::Protocol),
            ("WebRTC", SkillDomain::Protocol),
            ("SSE", SkillDomain::Protocol),
            ("Server-Sent Events", SkillDomain::Protocol),
            ("IPFS", SkillDomain::Protocol),
            ("GraphQL Federation", SkillDomain::Protocol),
            ("OpenAPI", SkillDomain::Protocol),
            ("AsyncAPI", SkillDomain::Protocol),
            ("GRPC-Web", SkillDomain::Protocol),
        ]
    }
}

impl Default for SkillDictionary {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Extraction Results ──────────────────────────────────────────────────────

/// Enriched intelligence extracted from a resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeIntelligence {
    /// Known tech skills found in the resume (with domain classification).
    pub known_skills: Vec<KnownSkill>,
    /// Skills inferred from context patterns (not in the dictionary but likely tech).
    pub inferred_skills: Vec<String>,
    /// Role/career level (e.g., "Senior Software Engineer", "Data Scientist").
    pub role_titles: Vec<String>,
    /// Seniority level inferred from titles.
    pub seniority: Option<SeniorityLevel>,
    /// Years of professional experience.
    pub experience_years: Option<f32>,
    /// Preferred location.
    pub preferred_location: Option<String>,
    /// Preferred job type.
    pub preferred_job_type: Option<String>,
    /// Domain categories detected (e.g., Backend, Frontend, DataMl).
    pub domains: Vec<SkillDomain>,
    /// Degrees extracted.
    pub education: Vec<Education>,
    /// Certifications extracted.
    pub certifications: Vec<String>,
    /// Meaningful keywords (tech terms + domain terms, filtered).
    pub keywords: Vec<String>,
}

/// Seniority level inferred from role titles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeniorityLevel {
    Intern,
    Junior,
    Mid,
    Senior,
    Staff,
    Principal,
    Lead,
    Director,
    Executive,
}

impl std::fmt::Display for SeniorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Intern => write!(f, "Intern"),
            Self::Junior => write!(f, "Junior"),
            Self::Mid => write!(f, "Mid-level"),
            Self::Senior => write!(f, "Senior"),
            Self::Staff => write!(f, "Staff"),
            Self::Principal => write!(f, "Principal"),
            Self::Lead => write!(f, "Lead"),
            Self::Director => write!(f, "Director"),
            Self::Executive => write!(f, "Executive"),
        }
    }
}

/// Education entry extracted from resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub degree: Option<String>,
    pub field: Option<String>,
    pub institution: Option<String>,
}

impl std::fmt::Display for Education {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = [&self.degree, &self.field, &self.institution]
            .iter()
            .filter_map(|s| s.as_ref().filter(|s| !s.is_empty()))
            .cloned()
            .collect();
        if parts.is_empty() {
            write!(f, "(unknown)")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

// ─── Parser ──────────────────────────────────────────────────────────────────

/// Parse raw resume text into structured intelligence.
pub fn parse_resume(text: &str) -> ResumeIntelligence {
    let dict = SkillDictionary::new();
    let lower = text.to_lowercase();

    // 1. Find known tech skills (the dictionary-driven approach)
    let known_skills = dict.find_skills(text);

    // 2. Find context-inferred skills (patterns like "experience with X")
    let inferred_skills = extract_context_skills(&lower, &dict);

    // 3. Extract role titles
    let role_titles = extract_roles(&lower);

    // 4. Infer seniority
    let seniority = infer_seniority(&role_titles, &lower);

    // 5. Extract experience years
    let experience_years = extract_experience_years(&lower);

    // 6. Extract location
    let preferred_location = extract_location(&lower);

    // 7. Extract job type
    let preferred_job_type = extract_job_type(&lower);

    // 8. Detect domains
    let domains = detect_domains(&known_skills);

    // 9. Extract education
    let education = extract_education(&lower);

    // 10. Extract certifications
    let certifications = extract_certifications(&lower);

    // 11. Build meaningful keywords (filtered to be useful)
    let keywords = build_keywords(&known_skills, &role_titles, &inferred_skills, &domains, &lower);

    ResumeIntelligence {
        known_skills: known_skills
            .iter()
            .map(|ks| KnownSkill {
                name: ks.name.clone(),
                domain: ks.domain,
            })
            .collect(),
        inferred_skills,
        role_titles,
        seniority,
        experience_years,
        preferred_location,
        preferred_job_type,
        domains,
        education,
        certifications,
        keywords,
    }
}

/// Extract skills from context patterns (for skills not in the dictionary).
fn extract_context_skills(text: &str, dict: &SkillDictionary) -> Vec<String> {
    let mut skills: Vec<String> = Vec::new();

    // Pattern: "experience with X", "proficient in X", "knowledge of X", etc.
    let ctx_re = Regex::new(
        r"(?i)(?:experience|proficient|skills?|knowledge|worked|familiar|expertise|strong|fluent|background|expert|using|including|exposure|hands.?on|practical|extensive|solid|deep|working)\s*(?:with|in|at|using|of|on|including)\s+([a-z][a-z+#.\d]+(?:\s+[a-z][a-z+#.\d]+)?)"
    ).unwrap();

    for cap in ctx_re.captures_iter(text) {
        let s = cap.get(1).unwrap().as_str().trim().to_string();
        if s.len() >= 2 && !dict.all_skill_names().contains(&s.to_lowercase()) {
            // Only include if it looks like a tech term (not a generic word)
            if looks_like_tech_term(&s) {
                skills.push(s);
            }
        }
    }

    // Pattern: "Technologies: X, Y, Z"
    let list_re = Regex::new(
        r"(?i)(?:technolog(y|ies)|tools?|tech\s*stack|languages?|frameworks?|platforms?|skills?|expertise)[:\s]+(.+)"
    ).unwrap();

    if let Some(caps) = list_re.captures(text) {
        let list_part = caps.get(2).unwrap().as_str();
        for item in list_part.split([',', ';', '|', '\n']).filter_map(|s| {
            let t = s.trim().trim_matches(|c: char| c == '.' || c == ' ').to_string();
            (t.len() >= 2 && !t.contains(char::is_whitespace)).then_some(t)
        }) {
            if !dict.all_skill_names().contains(&item.to_lowercase()) && looks_like_tech_term(&item) {
                skills.push(item);
            }
        }
    }

    skills.sort_unstable();
    skills.dedup();
    skills
}

/// Heuristic: does a term look like a tech skill (not a generic English word)?
fn looks_like_tech_term(term: &str) -> bool {
    let generic_tech_adjacent = [
        "development", "engineering", "software", "hardware", "programming",
        "coding", "implementation", "integration", "deployment", "testing",
        "analysis", "design", "architecture", "management", "operations",
        "administration", "support", "maintenance", "optimization", "automation",
        "configuration", "monitoring", "troubleshooting", "debugging",
        "framework", "library", "platform", "infrastructure", "pipeline",
        "workflow", "pipeline", "toolchain", "middleware", "frontend",
        "backend", "fullstack", "full-stack", "database", "analytics",
        "insights", "visualization", "reporting", "dashboard",
    ];

    let lower = term.to_lowercase();
    if generic_tech_adjacent.contains(&lower.as_str()) {
        return true;
    }

    // Check for version numbers, camelCase, special chars
    lower.contains(|c: char| c.is_ascii_digit() || c == '+' || c == '#' || c == '.')
        || lower.contains("js")
        || lower.contains("ql")
        || lower.chars().filter(|c| c.is_uppercase()).count() >= 2
}

/// Extract role titles from resume text.
fn extract_roles(text: &str) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();

    // "Role: Senior Engineer", "Position: Lead Developer"
    let ctx_re = Regex::new(
        r"(?i)(?:role|position|title|current|designation)[:\s]+([a-z]+(?:\s+[a-z]+){1,4})"
    ).unwrap();
    for cap in ctx_re.captures_iter(text) {
        let role = cap.get(1).unwrap().as_str().trim().to_string();
        if role.len() >= 5 {
            roles.push(role);
        }
    }

    // Full role titles: "Senior Software Engineer", "Frontend Developer", etc.
    let title_re = Regex::new(
        r"(?i)(?:(?:senior|staff|principal|lead|junior|intern|head|vp|vice\s+president|director|manager)\s+)?(?:software|data|full.?stack|frontend|backend|devops|platform|security|systems|network|site\s*reliability|machine\s*learning|ai|ml|cloud|infrastructure|product|program|project|technical|solution|support|qa|quality|test|embedded|firmware|mobile|ios|android|blockchain|research|applied|analytics|solutions|engineering|site\s*reliability|sre)\s+(?:engineer|developer|architect|manager|scientist|analyst|designer|director|lead|intern|specialist|consultant|coordinator|associate|administrator|programmer)"
    ).unwrap();
    for cap in title_re.captures_iter(text) {
        let role = cap.get(0).unwrap().as_str().trim().to_string();
        if !roles.contains(&role) {
            roles.push(role);
        }
    }

    roles.sort_unstable();
    roles.dedup();
    roles
}

/// Infer seniority level from role titles and text.
fn infer_seniority(roles: &[String], text: &str) -> Option<SeniorityLevel> {
    let combined = format!("{} {}", roles.join(" "), text).to_lowercase();

    if combined.contains("intern") {
        return Some(SeniorityLevel::Intern);
    }
    if combined.contains("junior") || combined.contains("jr") {
        return Some(SeniorityLevel::Junior);
    }
    if combined.contains("principal") || combined.contains("principle") {
        return Some(SeniorityLevel::Principal);
    }
    if combined.contains("staff") {
        return Some(SeniorityLevel::Staff);
    }
    if combined.contains("director") {
        return Some(SeniorityLevel::Director);
    }
    if combined.contains("vp ") || combined.contains("vice president") || combined.contains("chief") || combined.contains("cto") || combined.contains("ceo") {
        return Some(SeniorityLevel::Executive);
    }
    if combined.contains("lead") {
        return Some(SeniorityLevel::Lead);
    }
    if combined.contains("senior") || combined.contains("sr ") {
        return Some(SeniorityLevel::Senior);
    }

    // Default to mid-level if no indicator found
    None
}

/// Extract years of experience.
fn extract_experience_years(text: &str) -> Option<f32> {
    // Primary: "N years of experience", "N+ years exp"
    let re = Regex::new(r"(?i)(\d{1,2})\+?\s*(?:years?|yrs?)\s*(?:of\s+)?(?:experience|exp)").unwrap();
    if let Some(cap) = re.captures(text) {
        if let Ok(y) = cap.get(1)?.as_str().parse::<f32>() {
            if y > 0.0 && y < 50.0 {
                return Some(y);
            }
        }
    }

    // Fallback: "N+ years" in experience section context
    let re2 = Regex::new(r"(?i)(?:over|>|more\s+than)\s*(\d{1,2})\+?\s*years?").unwrap();
    if let Some(cap) = re2.captures(text) {
        if let Ok(y) = cap.get(1)?.as_str().parse::<f32>() {
            if y > 0.0 && y < 50.0 {
                return Some(y);
            }
        }
    }

    None
}

/// Extract preferred location.
fn extract_location(text: &str) -> Option<String> {
    let re = Regex::new(
        r"(?i)(?:based|located|living|situated|relocated|willing\s+to\s+relocate\s+to)\s+(?:in|at|near|to)\s+([a-z][a-z\s.-]{2,40}?)(?:[,.!]|$)"
    ).unwrap();
    re.captures(text).and_then(|c| {
        let loc = c.get(1)?.as_str().trim().to_string();
        (loc.len() <= 50 && loc.len() >= 2).then_some(loc)
    })
}

/// Extract preferred job type.
fn extract_job_type(text: &str) -> Option<String> {
    let re = Regex::new(
        r"(?i)(?:looking|seeking|want|prefer|open|available|interested)\s+(?:for|a|an|in)?\s*(full[- ]time|part[- ]time|contract|remote|hybrid|onsite|internship)"
    ).unwrap();
    if let Some(cap) = re.captures(text) {
        return cap.get(1).map(|m| m.as_str().to_string());
    }

    // Fallback: scan for job type keywords in preference context
    let section = Regex::new(r"(?i)(?:type|preference|work\s+mode)[:\s]+([^\\n]+)").unwrap();
    if let Some(cap) = section.captures(text) {
        let s = cap.get(1).unwrap().as_str().to_lowercase();
        for t in &["remote", "hybrid", "onsite", "full-time", "full time", "part-time", "part time", "contract", "internship"] {
            if s.contains(t) {
                return Some(t.to_string());
            }
        }
    }

    None
}

/// Detect domains from known skills.
fn detect_domains(known_skills: &[KnownSkill]) -> Vec<SkillDomain> {
    let mut domains: Vec<SkillDomain> = known_skills
        .iter()
        .map(|s| s.domain)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    domains.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    domains
}

/// Extract education entries.
fn extract_education(text: &str) -> Vec<Education> {
    let mut education = Vec::new();

    // Degree patterns
    let degree_re = Regex::new(
        r"(?i)((?:bachelor|master|ph\.?d|doctorate|associate|b\.?s\.?|m\.?s\.?|b\.?a\.?|m\.?a\.?|phd|bs|ba|ms|ma|b\.?tech|m\.?tech)\s+(?:of|in|\.)?\s*[a-z\s]{2,40}?(?:degree)?)"
    ).unwrap();

    for cap in degree_re.captures_iter(text) {
        let deg = cap.get(1).unwrap().as_str().trim().to_string();
        if deg.len() > 3 && deg.len() < 60 {
            let edu = Education {
                degree: Some(deg),
                field: None,
                institution: None,
            };
            education.push(edu);
        }
    }

    // Try to extract institution names from nearby text
    let institution_re = Regex::new(
        r"(?i)((?:university|college|institute|school)\s+of\s+[a-z\s]{2,40}|[a-z]+\s+(?:university|college|institute))"
    ).unwrap();

    if let Some(cap) = institution_re.captures(text) {
        let inst = cap.get(1).unwrap().as_str().trim().to_string();
        if let Some(edu) = education.last_mut() {
            edu.institution = Some(inst);
        }
    }

    education
}

/// Extract certifications.
fn extract_certifications(text: &str) -> Vec<String> {
    let mut certs = Vec::new();

    let patterns = [
        r"(?i)(?:certified|certification|certificate|credential)\s+(?:in|:)?\s*([a-z][a-z\s]{2,50}(?:certification)?)",
        r"(?i)([a-z][a-z\s]{2,40})\s+(?:certification|certificate|credential)",
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(text) {
                let cert = cap.get(1).unwrap().as_str().trim().to_string();
                if cert.len() > 3 && !certs.contains(&cert) {
                    certs.push(cert);
                }
            }
        }
    }

    // Known certifications (case-insensitive match)
    let known_certs = [
        "aws certified", "aws certification", "azure certified", "azure certification",
        "gcp certified", "gcp certification", "google cloud certified",
        "cka", "ckad", "cks", "cissp", "ceh", "oscp", "pmp", "scrum master",
        "csm", "psm", "safe", "itil", "comptia", "ccna", "ccnp", "ccie",
        "ocp", "ocjp", "ocajp", "rhcsa", "rhce", "lfcs", "cfe", "cfa",
        "frm", "prince2", "six sigma", "lean", "cobit", "togaf",
    ];

    for cert in &known_certs {
        if text.to_lowercase().contains(cert) {
            certs.push(cert.to_string());
        }
    }

    certs.sort_unstable();
    certs.dedup();
    certs
}

/// Build meaningful keywords — filtered to be useful for job matching.
fn build_keywords(
    known_skills: &[KnownSkill],
    role_titles: &[String],
    inferred_skills: &[String],
    domains: &[SkillDomain],
    text: &str,
) -> Vec<String> {
    let mut keywords: Vec<String> = Vec::new();

    // 1. All known skill names
    for ks in known_skills {
        keywords.push(ks.name.to_lowercase());
    }

    // 2. All role titles (as multi-word keywords)
    for role in role_titles {
        keywords.push(role.to_lowercase());
        // Also add individual significant words from roles
        for word in role.split_whitespace() {
            let w = word.to_lowercase();
            if w.len() >= 3 && !is_junk_word(&w) {
                keywords.push(w);
            }
        }
    }

    // 3. Inferred skills
    for s in inferred_skills {
        keywords.push(s.to_lowercase());
    }

    // 4. Domain-specific keywords
    for domain in domains {
        let domain_kws = match domain {
            SkillDomain::Language => vec!["programming", "software development"],
            SkillDomain::Frontend => vec!["frontend", "web development", "ui", "ux"],
            SkillDomain::Backend => vec!["backend", "api", "server", "service"],
            SkillDomain::Database => vec!["database", "data storage", "query"],
            SkillDomain::CloudDevOps => vec!["cloud", "devops", "infrastructure", "deployment"],
            SkillDomain::DataMl => vec!["data", "machine learning", "analytics"],
            SkillDomain::Mobile => vec!["mobile", "app development"],
            SkillDomain::Tools => vec![],
            SkillDomain::Platform => vec![],
            SkillDomain::Protocol => vec![],
            SkillDomain::Concept => vec![],
            SkillDomain::Framework => vec![],
            SkillDomain::Other => vec![],
        };
        for kw in domain_kws {
            if !keywords.contains(&kw.to_string()) {
                keywords.push(kw.to_string());
            }
        }
    }

    // 5. Experience-seniority keywords
    let seniority_kws = ["senior", "staff", "principal", "lead", "junior", "experienced"];
    for sk in &seniority_kws {
        if text.contains(sk) && !keywords.contains(&sk.to_string()) {
            keywords.push(sk.to_string());
        }
    }

    keywords.sort_unstable();
    keywords.dedup();
    keywords
}

/// Check if a word is junk (common stop word, not useful for matching).
fn is_junk_word(word: &str) -> bool {
    let junk = [
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
        "was", "one", "our", "out", "has", "have", "been", "some", "same",
        "its", "than", "them", "into", "two", "more", "these", "like", "over",
        "such", "that", "this", "with", "from", "your", "which", "each", "will",
        "about", "between", "under", "very", "just", "their", "would", "after",
        "could", "should", "than", "then", "there", "where", "while", "because",
        "before", "does", "doing", "done", "much", "many", "most", "must",
        "need", "take", "make", "made", "well", "work", "year", "years",
        "also", "back", "still", "get", "use", "used", "using", "way", "new",
        "first", "last", "own", "see", "say", "said", "may", "though",
    ];
    junk.contains(&word)
}
