import source from "./assets/hyperbilling-logo.svg?raw";

const CANVAS = 'width="580" height="150" viewBox="0 0 580 150"';

// "MCP" drawn to the cap height and stem weight of the wordmark's bold half.
const SUFFIX = `<path d="M540 97.4468V38.3201H555.5L571 76L586.5 38.3201H602V97.4468H590.4V58L575.2 97.4468H566.8L551.6 58V97.4468Z" fill="white"/>
<path d="M656.75 49.11A24.5 24.5 0 1 0 656.75 86.65" stroke="white" stroke-width="12.08"/>
<path fill-rule="evenodd" d="M668 97.4468V38.3201H692.5C706 38.3201 714 46 714 57.3C714 68.6 706 76.3 692.5 76.3H680.08V97.4468ZM680.08 65.9H691C697.6 65.9 701.6 62.6 701.6 57.3C701.6 52 697.6 48.7 691 48.7H680.08Z" fill="white"/>
</svg>`;

// The vendor file is white on a fixed canvas with wide margins.
const paint = (svg: string) =>
  svg.replaceAll('fill="white"', 'fill="currentColor"').replaceAll('stroke="white"', 'stroke="currentColor"');

const markup = paint(
  source
    .trim()
    .replace(CANVAS, 'class="logo" viewBox="62 33 457 84" role="img" aria-label="HyperBilling"'),
);

const brandMarkup = paint(
  source
    .trim()
    .replace(CANVAS, 'class="logo" viewBox="62 33 654 84" aria-hidden="true"')
    .replace(/<\/svg>\s*$/, SUFFIX),
);

export function Logo() {
  return <span className="logo-wrap" dangerouslySetInnerHTML={{ __html: markup }} />;
}

export function BrandLogo() {
  return <span className="logo-wrap" dangerouslySetInnerHTML={{ __html: brandMarkup }} />;
}
