import source from "./assets/hyperbilling-logo.svg?raw";

// The vendor file is white on a fixed canvas with wide margins.
const markup = source
  .trim()
  .replace(
    'width="580" height="150" viewBox="0 0 580 150"',
    'class="logo" viewBox="62 33 457 84" role="img" aria-label="HyperBilling"',
  )
  .replaceAll('fill="white"', 'fill="currentColor"');

export function Logo() {
  return <span className="logo-wrap" dangerouslySetInnerHTML={{ __html: markup }} />;
}
