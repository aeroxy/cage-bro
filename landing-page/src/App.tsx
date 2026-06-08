import { SiteHeader } from '@/components/site-header'
import { Hero } from '@/components/hero'
import { IntroSection } from '@/components/intro-section'
import { PhilosophySection } from '@/components/philosophy-section'
import { JailedSection } from '@/components/jailed-section'
import { FleetSection } from '@/components/fleet-section'
import { VmSection } from '@/components/vm-section'
import { SecuritySection } from '@/components/security-section'
import { ReferenceSection } from '@/components/reference-section'
import { SiteFooter } from '@/components/site-footer'

export default function App() {
  return (
    <div id="top" className="min-h-screen bg-background">
      <SiteHeader />
      <main>
        <Hero />
        <IntroSection />
        <PhilosophySection />
        <JailedSection />
        <FleetSection />
        <VmSection />
        <SecuritySection />
        <ReferenceSection />
      </main>
      <SiteFooter />
    </div>
  )
}
