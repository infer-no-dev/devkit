# DevKit Examples

A growing collection of end-to-end examples you can copy and adapt.

- Small business website (Next.js + TypeScript + Tailwind)
- (Coming soon) Rust API with Axum + SQLx
- (Coming soon) Python FastAPI service

---

## Small business website (Next.js + TypeScript + Tailwind)

Goal: scaffold and run a marketing website for a local home maintenance & repair business. Includes pages, components, SEO, and a contact form endpoint you can wire to email.

### 1) Prerequisites (Node LTS)

```bash
# Install nvm + LTS Node (Ubuntu)
[ -d "$HOME/.nvm" ] || curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
export NVM_DIR="$HOME/.nvm" && . "$NVM_DIR/nvm.sh"
command -v node >/dev/null 2>&1 || nvm install --lts
node -v && npm -v
```

### 2) Plan the scaffold (dry-run)

Describe the site in a prompt. This example uses “HomeCare Pros.”

```bash
# From your repo root
devkit generate "HomeCare Pros website: Next.js, TS, Tailwind. Pages: Home, Services (Appliance Repair, Plumbing, Electrical, Carpentry), About, Contact. Components: Header, Footer, Hero, ServiceCard, Testimonials, FAQ. SEO (title/description, OG tags), sitemap, robots.txt. Contact page with form (POST /api/contact). Responsive, accessible, clean design." \
  --language typescript \
  --stack nextjs \
  --root ./web/homecare-pros \
  --dry-run \
  --export-plan ./web-plan.json
```

- This prints a plan and writes `web-plan.json`. Re-run with revised prompts until the plan looks right.

### 3) Apply the scaffold plan

Note: `--apply-plan` currently requires a (dummy) prompt argument.

```bash
devkit generate "apply plan: HomeCare Pros website" --apply-plan ./web-plan.json --force
```

### 4) Install and run locally

```bash
cd ./web/homecare-pros
npm install
npm run dev
# open http://localhost:3000
```

### 5) Customize content & branding

- Replace copy and contact details on each page.
- Swap the logo and images under `public/`.
- Update default metadata (title/description) and OpenGraph tags.

### 6) Contact form wiring (email)

The scaffold includes a `/api/contact` route. Configure SMTP in `.env.local` and update the handler if needed (e.g., using `nodemailer`).

```bash
# ./web/homecare-pros/.env.local
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=your_user
SMTP_PASS=your_password
CONTACT_TO=owner@example.com
```

### 7) SEO essentials

- Ensure page-level metadata (title, description) and OpenGraph tags (og:title, og:description, og:image).
- Add `sitemap.xml` and `robots.txt` if not already present.
- Add JSON-LD for LocalBusiness on the Home page (name, address, telephone, openingHours). You can ask DevKit to create or update this.

### 8) Iterate quickly with DevKit chat

Use the chat to add sections and pages without leaving the terminal.

```bash
# Start chat
devkit chat --role developer

# Examples (type inside chat):
# Add a Testimonials section to the Home page with 3 quotes
# Create a Plumbing service subpage with pricing and a CTA
# Add LocalBusiness JSON-LD to the Home page
```

### 9) Deploy

Vercel is simplest for Next.js:

```bash
npm i -g vercel
vercel
```

Alternatively, push to GitHub and connect the repository to Vercel’s dashboard.

### 10) Checklist

- Mobile responsiveness (narrow viewport test)
- Lighthouse (Performance/Accessibility/Best Practices/SEO > 90)
- Accessibility (focus states, alt text, aria labels, contrast)
- Forms (client + server-side validation, success/error messaging)
- Analytics snippet (e.g., GA4) and conversion event on form submit

---

Need another example added? Open an issue or ask in DevKit chat (e.g., “add a Rust Axum API example to docs/EXAMPLES.md”).
