-- Packages table
CREATE TABLE IF NOT EXISTS packages (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    repository TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);

-- Package versions table
CREATE TABLE IF NOT EXISTS package_versions (
    id BIGSERIAL PRIMARY KEY,
    package_id BIGINT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    license TEXT,
    checksum TEXT NOT NULL,
    tarball_s3_key TEXT NOT NULL,
    tarball_size BIGINT NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL,
    downloads BIGINT DEFAULT 0,
    FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE,
    UNIQUE(package_id, version)
);

CREATE INDEX IF NOT EXISTS idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX IF NOT EXISTS idx_package_versions_version ON package_versions(version);

-- Package authors table
CREATE TABLE IF NOT EXISTS package_authors (
    id BIGSERIAL PRIMARY KEY,
    package_version_id BIGINT NOT NULL,
    author TEXT NOT NULL,
    FOREIGN KEY (package_version_id) REFERENCES package_versions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_package_authors_version_id ON package_authors(package_version_id);

-- Package keywords table
CREATE TABLE IF NOT EXISTS package_keywords (
    id BIGSERIAL PRIMARY KEY,
    package_id BIGINT NOT NULL,
    keyword TEXT NOT NULL,
    FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_package_keywords_package_id ON package_keywords(package_id);
CREATE INDEX IF NOT EXISTS idx_package_keywords_keyword ON package_keywords(keyword);

-- Package dependencies table
CREATE TABLE IF NOT EXISTS package_dependencies (
    id BIGSERIAL PRIMARY KEY,
    package_version_id BIGINT NOT NULL,
    dependency_name TEXT NOT NULL,
    version_constraint TEXT NOT NULL,
    FOREIGN KEY (package_version_id) REFERENCES package_versions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_package_dependencies_version_id ON package_dependencies(package_version_id);
CREATE INDEX IF NOT EXISTS idx_package_dependencies_name ON package_dependencies(dependency_name);

-- Download statistics table
CREATE TABLE IF NOT EXISTS download_stats (
    id BIGSERIAL PRIMARY KEY,
    package_version_id BIGINT NOT NULL,
    downloaded_at TIMESTAMP WITH TIME ZONE NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (package_version_id) REFERENCES package_versions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_download_stats_version_id ON download_stats(package_version_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_downloaded_at ON download_stats(downloaded_at);
