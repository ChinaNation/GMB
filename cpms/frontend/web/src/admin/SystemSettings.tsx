import Qr2Generate from './Qr2Generate';
import AnonCertScan from './AnonCertScan';

export default function SystemSettings() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <Qr2Generate />
      <AnonCertScan />
    </div>
  );
}
