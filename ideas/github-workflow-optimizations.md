# GitHub Workflow Optimization Ideas

This document contains optimization strategies to speed up the GitHub Actions release workflow for MyTodos.

## Current Workflow Analysis

**File:** `.github/workflows/release.yml`

**Current Setup:**
- Builds for 4 platforms: macOS (arm64), macOS (x86_64), Ubuntu 22.04, Windows
- Uses matrix strategy for parallel builds ✅
- Has Rust caching ✅
- Missing npm caching ❌

## Optimization Suggestions

### 🚀 High Impact (Quick Wins)

#### 1. Cache npm dependencies
**Impact:** Saves 30-60 seconds per build
**Effort:** Minimal (1 line change)
**Status:** Not implemented

```yaml
- name: Setup Node.js
  uses: actions/setup-node@v4
  with:
    node-version: 20
    cache: 'npm'  # Add this line
```

**Why it helps:** npm install downloads and installs all dependencies from scratch every time. Caching saves significant time on repeated builds.

---

#### 2. Remove unnecessary apt-get update
**Impact:** Saves 10-20 seconds on Ubuntu builds
**Effort:** Minimal (delete 1 line)
**Status:** Not implemented

**Current:**
```yaml
- name: Install dependencies (Ubuntu only)
  if: matrix.platform == 'ubuntu-22.04'
  run: |
    sudo apt-get update  # Remove this line
    sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Optimized:**
```yaml
- name: Install dependencies (Ubuntu only)
  if: matrix.platform == 'ubuntu-22.04'
  run: |
    sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Why it helps:** GitHub Actions runners are pre-configured and up-to-date. The update step is usually unnecessary and just adds overhead.

---

#### 3. Optimize Rust cache configuration
**Impact:** Better cache hit rates, faster subsequent builds
**Effort:** Low (2-3 line change)
**Status:** Not implemented

```yaml
- name: Rust cache
  uses: swatinem/rust-cache@v2
  with:
    workspaces: './src-tauri -> target'
    key: ${{ matrix.platform }}-${{ matrix.args }}  # Platform-specific cache
    shared-key: "release"  # Share cache across runs
    save-if: ${{ github.ref_name == 'main' }}  # Only save cache on main branch
```

**Why it helps:**
- Platform-specific keys prevent cache misses when switching architectures
- Shared key allows cache reuse across different builds
- Conditional save reduces cache churn

---

### ⚡ Medium Impact (High Value)

#### 4. Switch to pnpm
**Impact:** Saves 40-70% on dependency installation time
**Effort:** Medium (requires lockfile conversion, ~10 min)
**Status:** Not implemented

**Setup:**
```yaml
- name: Install pnpm
  uses: pnpm/action-setup@v2
  with:
    version: 8

- name: Setup Node.js
  uses: actions/setup-node@v4
  with:
    node-version: 20
    cache: 'pnpm'

- name: Install frontend dependencies
  run: pnpm install --frozen-lockfile
```

**Migration steps:**
1. Install pnpm locally: `npm install -g pnpm`
2. Import from package-lock.json: `pnpm import`
3. Commit `pnpm-lock.yaml`
4. Update workflow file
5. Update README if needed

**Why it helps:**
- pnpm uses a content-addressable store, sharing dependencies across projects
- Much faster installation and less disk space
- Still compatible with npm packages

---

#### 5. Add sccache for Rust compilation
**Impact:** Saves 2-5 minutes on Rust compilation (especially on cache miss)
**Effort:** Medium (~5-10 min setup)
**Status:** Not implemented

```yaml
- name: Install sccache
  uses: mozilla-actions/sccache-action@v0.0.4

- name: Setup Rust with sccache
  run: |
    echo "RUSTC_WRAPPER=sccache" >> $GITHUB_ENV
    echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV

- name: Install Rust stable
  uses: dtolnay/rust-toolchain@stable
  with:
    targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}
```

**Why it helps:**
- sccache caches compiled Rust artifacts across builds
- Works even when `Cargo.lock` changes
- Particularly helpful for incremental builds

---

### 💰 Paid Optimizations

#### 6. Use larger GitHub runners
**Impact:** 2-4x faster builds
**Cost:** $0.008/minute (4-core), $0.016/minute (8-core) vs $0.008/minute (2-core)
**Effort:** Low (1 line change per platform)
**Status:** Not implemented

```yaml
runs-on: ${{ matrix.platform == 'ubuntu-22.04' && 'ubuntu-22.04-4-cores' ||
           matrix.platform == 'windows-latest' && 'windows-latest-8-cores' ||
           matrix.platform }}
```

**Cost analysis:**
- Current: ~15 min/build × 4 platforms × $0.008/min = ~$0.48/release
- With 4-core: ~8 min/build × 4 platforms × $0.008/min = ~$0.26/release
- With 8-core: ~5 min/build × 4 platforms × $0.016/min = ~$0.32/release

**Best for:**
- Frequent releases
- Large projects with long build times
- When developer time is more valuable than runner cost

---

#### 7. Self-hosted runners
**Impact:** Potentially 5-10x faster with persistent caches
**Cost:** Server/hardware costs + maintenance
**Effort:** High (requires setup and ongoing maintenance)
**Status:** Not recommended for this project size

**Why it helps:**
- Persistent caches across all builds
- Full control over hardware specs
- No GitHub Actions usage costs

**Downsides:**
- Security concerns (need to carefully manage credentials)
- Maintenance overhead
- Upfront infrastructure costs
- Probably overkill for a single small project

---

### 🔧 Advanced Optimizations

#### 8. Split frontend/backend builds
**Impact:** Moderate (parallel work, better cache granularity)
**Effort:** Medium-High (workflow restructuring)
**Status:** Not implemented

**Concept:**
```yaml
jobs:
  build-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'npm'
      - run: npm ci
      - run: npm run build
      - uses: actions/upload-artifact@v4
        with:
          name: frontend-build
          path: build/

  build-tauri:
    needs: build-frontend
    strategy:
      matrix:
        # ... existing matrix
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          name: frontend-build
          path: build/
      # ... rest of Tauri build
```

**Why it helps:**
- Frontend builds once instead of 4 times
- Better separation of concerns
- Easier to debug build issues

**Complexity:**
- Requires careful artifact management
- More workflow configuration
- May not save much time if frontend build is fast

---

#### 9. Conditional platform builds
**Impact:** Saves entire platform build times
**Effort:** Low (conditional expressions)
**Status:** Not implemented

```yaml
strategy:
  matrix:
    include:
      - platform: 'macos-latest'
        args: '--target aarch64-apple-darwin'
        # Build macOS on all releases
      - platform: 'macos-latest'
        args: '--target x86_64-apple-darwin'
        # Build macOS on all releases
      - platform: 'ubuntu-22.04'
        args: ''
        if: ${{ github.event_name == 'workflow_dispatch' || contains(github.ref, 'rc') }}
        # Build Linux only on manual trigger or RC tags
      - platform: 'windows-latest'
        args: ''
        # Build Windows on all releases
```

**Use cases:**
- Skip certain platforms for pre-release/RC builds
- Only build all platforms for stable releases
- Useful if you primarily develop/test on one platform

---

#### 10. Use Turbo or Nx for monorepo caching
**Impact:** High (if project grows into monorepo)
**Effort:** High
**Status:** Not applicable (not a monorepo)

**Future consideration:** If you add mobile apps, desktop variants, or other packages.

---

## Implementation Roadmap

### Phase 1: Quick Wins (1 hour)
**Implement now for immediate gains:**
- ✅ Add npm caching (Optimization #1)
- ✅ Remove apt-get update (Optimization #2)
- ✅ Optimize Rust cache (Optimization #3)

**Expected savings:** 1-2 minutes per build

---

### Phase 2: High Value (2-3 hours)
**Implement next for major improvements:**
- ⬜ Switch to pnpm (Optimization #4)
- ⬜ Add sccache (Optimization #5)

**Expected savings:** 3-7 minutes per build

---

### Phase 3: Consider Later
**Evaluate based on project needs:**
- ⬜ Larger runners (Optimization #6) - if building frequently
- ⬜ Split frontend/backend (Optimization #8) - if frontend gets complex
- ⬜ Conditional builds (Optimization #9) - if release cadence increases

---

### Phase 4: Don't Do (Yet)
**Not worth it at current project scale:**
- ❌ Self-hosted runners (Optimization #7) - too much overhead
- ❌ Monorepo tools (Optimization #10) - not a monorepo

---

## Expected Results

### Current baseline (estimated)
- **Cold cache:** ~15-20 minutes per platform
- **Warm cache:** ~10-15 minutes per platform
- **Total release time:** ~40-60 minutes (parallel)

### After Phase 1 optimizations
- **Cold cache:** ~14-18 minutes per platform (-5-10%)
- **Warm cache:** ~8-12 minutes per platform (-20-30%)
- **Total release time:** ~32-48 minutes

### After Phase 2 optimizations
- **Cold cache:** ~12-15 minutes per platform (-15-25%)
- **Warm cache:** ~5-8 minutes per platform (-50-60%)
- **Total release time:** ~20-32 minutes

### With paid runners (Phase 3)
- **Cold cache:** ~6-8 minutes per platform (-60-70%)
- **Warm cache:** ~3-5 minutes per platform (-70-80%)
- **Total release time:** ~12-20 minutes

---

## Monitoring & Validation

After implementing optimizations, track:
- Build duration per platform (GitHub Actions UI)
- Cache hit rate (check workflow logs)
- Total release time
- Any build failures or issues

**Useful metrics dashboard:**
- Go to repository → Actions → Click workflow
- View "Usage this month" for cost tracking
- Check individual runs for timing breakdowns

---

## Additional Notes

### When NOT to optimize
- If you only release once a month or less
- If current build time is acceptable
- If you're approaching GitHub Actions free tier limits

### When TO optimize
- Releasing multiple times per week
- Rapid iteration during development
- Building on every PR (if you add CI workflow)
- Team productivity is impacted by slow builds

### Cache management
- GitHub keeps caches for 7 days or until 10GB limit
- Use workflow_run cleanup to manage cache size:
  ```yaml
  - name: Cleanup old caches
    uses: actions/github-script@v7
    with:
      script: |
        // Script to delete old caches
  ```

---

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Cache Action](https://github.com/swatinem/rust-cache)
- [sccache Documentation](https://github.com/mozilla/sccache)
- [pnpm Documentation](https://pnpm.io/)
- [Tauri Build Optimization](https://tauri.app/v1/guides/building/cross-platform#optimizing-builds)
- [GitHub Actions Larger Runners](https://docs.github.com/en/actions/using-github-hosted-runners/about-larger-runners)

---

**Last Updated:** 2026-01-05
**Status:** Documentation phase
**Next Action:** Implement Phase 1 optimizations when ready
