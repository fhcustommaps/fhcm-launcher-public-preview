static const char hidden_hud[]=R"FHCM(<Page xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
      xmlns:System="clr-namespace:System;assembly=mscorlib"
      xmlns:local="clr-namespace:ForzaUI;assembly="
      x:Class="ForzaUI.A_Hud"
      x:Name="Page"
      WindowTitle="Page"
      FlowDirection="LeftToRight"
      Style="{StaticResource A_Style_ScreenSize}"
      DataContext="{Binding RelativeSource={x:Static RelativeSource.Self}}">

    <Page.Resources>
        <ResourceDictionary>
            <!--
            <ResourceDictionary.MergedDictionaries>
                <ResourceDictionary Source="pack://application:,,,/ForzaUI;component/Resources/Anthem/Shared_Hud.xaml" />
            </ResourceDictionary.MergedDictionaries>
            -->
        </ResourceDictionary>
    </Page.Resources>

    <local:UIPageTransitionManager.TransitionManager>
        <local:UIPageTransitionManager DurationSpread="{StaticResource A_Duration_Transition_Spread}"
                                       TransitionDelay="0" />
    </local:UIPageTransitionManager.TransitionManager>

    <local:ZPanel x:Name="LayoutRoot"
                  Style="{StaticResource A_Style_FrameworkElement_FadeOnlyTransition_0}">
        <Grid Opacity="0" IsHitTestVisible="False" Width="{StaticResource A_Double_SceneWidth}"
              Height="{StaticResource A_Double_SceneHeight}">

            <!-- screen space shadow -->
            <local:FZMaterial Preset="{StaticResource A_FZMaterialPresetScreenShadow_ARUI}" />

            <!-- HUD Controls -->
            <local:HUDActivationControl x:Name="HUDStatusActivation"
                                        Margin="110,0,0,250"
                                        VerticalAlignment="Center"
                                        HorizontalAlignment="Left" />

            <!--Always On-->
            <local:HUDAnnaControl x:Name="HUDStatusAnna"
                                  HorizontalAlignment="Left"
                                  VerticalAlignment="Bottom"
                                  Margin="54,0,0,92" />
            <local:HUDSpeedometerControl x:Name="HUDStatusSpeedometer"
                                         HorizontalAlignment="Right"
                                         VerticalAlignment="Bottom"
                                         Width="300"
                                         Height="300"
                                         Margin="0,0,92,23" />
            <local:HUDRadioControl x:Name="HUDStatusRadio"
                                   HorizontalAlignment="Right"
                                   VerticalAlignment="Bottom"
                                   Margin="0,0,118,350" />
            <local:HUDWaitSpinnerControl x:Name="HUDStatusWaitSpinner"
                                         HorizontalAlignment="Right"
                                         Margin="0,0,408,92"
                                         VerticalAlignment="Bottom"
                                         Width="32"
                                         Height="32" />
            <local:HUDSkillControl x:Name="HUDStatusSkill"
                                   Margin="0, -632, 0, 0"
                                   VerticalAlignment="Center"
                                   HorizontalAlignment="Center" />
            <local:HUDSkillTokenControl x:Name="HUDStatusSkillToken"
                                        Margin="0,-820,0,0"
                                        VerticalAlignment="Center"
                                        HorizontalAlignment="Center" />

            <!--SimpleStatusControls-->
            <local:HUDWrongWayControl x:Name="HUDStatusWrongWay"
                                      Margin="0,270,0,0"
                                      VerticalAlignment="Top" />
            <local:HUDPressButtonControl x:Name="HUDStatusPressButton"
                                         Margin="0,0,0,80"
                                         VerticalAlignment="Bottom"
                                         HorizontalAlignment="Center" />

            <!--Race Controls-->
            <local:HUD321Control x:Name="HUDStatus321"
                                 Margin="0,300,0,0"
                                 VerticalAlignment="Top"
                                 HorizontalAlignment="Center" />
            <local:HUDRaceProgressControl x:Name="HUDStatusRaceProgress"
                                          HorizontalAlignment="Left"
                                          VerticalAlignment="Top"
                                          Margin="108,86,0,0" />
            <local:HUDMiniLeaderboardControl x:Name="HUDStatusMiniLeaderboard"
                                             HorizontalAlignment="Right"
                                             VerticalAlignment="Top"
                                             Margin="0,71,70,0" />

            <!-- Notifications and Transient Controls -->
            <local:HUDInstructionControl x:Name="HUDStatusInstruction"
                                         Margin="108,86,0,0"
                                         Visibility="{Binding IsVisibilityOptionEnabled, Converter={StaticResource A_FZBooleanToVisibilityConverter}}"/>
            <local:HUDMajorNotificationControl x:Name="HUDStatusMajorNotification"
                                               Margin="0,270,0,0"
                                               VerticalAlignment="Top" />
            <local:HUDMinorNotificationControl x:Name="HUDStatusMinorNotification"
                                               Margin="0,0,0,130"
                                               VerticalAlignment="Bottom" />
            <local:HUDSpeedCameraControl x:Name="HUDStatusSpeedCamera"
                                         Margin="0,0,0,55"
                                         VerticalAlignment="Bottom" />
            <local:HUDRivalsEverywhereControl x:Name="HUDStatusRivalsEverywhere"
                                              Margin="0,0,0,130"
                                              VerticalAlignment="Bottom" />
            <local:HUDRaceEncounterControl x:Name="HUDStatusRaceEncounter"
                                           Margin="680.716,0,767.283,59"
                                           VerticalAlignment="Bottom" />
            <local:HUDXPAwardControl x:Name="HUDStatusXPAward"
                                     Margin="0,0,730,50"
                                     VerticalAlignment="Center"
                                     HorizontalAlignment="Center" />

            <!-- Network Controls -->
            <local:HUDNetworkStatusControl x:Name="HUDStatusNetworkStatus"
                                           Margin="0,-950,0,0"
                                           VerticalAlignment="Center"
                                           HorizontalAlignment="Center" />
            <local:HUDDriveToEventControl x:Name="HUDStatusNetworkRoadTripDestinationDrive"
                                          HorizontalAlignment="Left"
                                          VerticalAlignment="Top"
                                          Margin="105,79,0,0" />
            <local:HUDRaceFinishCountdownControl x:Name="HUDStatusRaceCountdown"
                                                 Margin="0,0,0,91"
                                                 HorizontalAlignment="Center"
                                                 VerticalAlignment="Bottom" />

            <local:HUDBucketListControl x:Name="HUDStatusBucketList"
                                        Margin="108,56,108,0"
                                        HorizontalAlignment="Right" />
        </Grid>
    </local:ZPanel>
</Page>
)FHCM";