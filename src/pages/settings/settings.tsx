import { Options } from './options';
import { BiRegularArrowBack as ArrowLeft } from 'solid-icons/bi';
import { createSignal } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import ThemeCustomization from './themes/theme-customization';
import { Button } from 'components/ui/button';
import { About } from './about';

type SettingsTab = 'theme' | 'options' | 'about';

export function Settings() {
  const [activeTab, setActiveTab] = createSignal<SettingsTab>('theme');
  const navigate = useNavigate();
  const handleBackClick = () => {
    navigate(-1);
  };

  return (
    <div class="min-h-screen bg-background flex items-center justify-center p-4">
      <div class="w-full max-w-6xl bg-card rounded-lg shadow-lg overflow-hidden flex">
        <div class="w-64 bg-muted p-4">
          <button
            onClick={handleBackClick}
            class="mb-4 flex items-center text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            <ArrowLeft class="mr-2 h-4 w-4" />
            Back
          </button>
          <h2 class="text-2xl font-bold mb-4 text-foreground">Settings</h2>
          <nav>
            <ul class="space-y-2">
              <li>
                <Button
                  class="w-full"
                  variant={activeTab() === 'theme' ? 'default' : 'ghost'}
                  onClick={() => setActiveTab('theme')}
                >
                  Theme Customization
                </Button>
              </li>
              <li>
                <Button
                  class="w-full"
                  variant={activeTab() === 'options' ? 'default' : 'ghost'}
                  onClick={() => setActiveTab('options')}
                >
                  Options
                </Button>
              </li>
              <li>
                <Button
                  class="w-full"
                  variant={activeTab() === 'about' ? 'default' : 'ghost'}
                  onClick={() => setActiveTab('about')}
                >
                  About
                </Button>
              </li>
            </ul>
          </nav>
        </div>

        {/* Content */}
        <div class="flex-1 flex overflow-hidden">
          <div class="flex-1 p-8 h-[600px] overflow-y-auto border">
            {activeTab() === 'theme' && <ThemeCustomization />}
            {activeTab() === 'options' && <Options />}
            {activeTab() === 'about' && <About />}
          </div>
        </div>
      </div>
    </div>
  );
}
